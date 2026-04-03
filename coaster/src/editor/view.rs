mod buffer;
mod line;

use std::{
    cmp::{max, min},
    io::Error,
};

use self::line::Line;
use super::{
    DocumentStatus, NAME, VERSION,
    editorcommand::{Direction, EditorCommand, Mode},
    terminal::{Position, Size, Terminal},
    uicomponent::UIComponent,
};
use buffer::Buffer;

const DEFAULT_LINE: &str = "~";

#[derive(Copy, Clone, Default)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

#[derive(Default)]
pub struct View {
    buffer: Buffer,
    pub needs_redrawn: bool,
    size: Size,
    line_padding: usize,
    text_location: Location,
    scroll_offset: Position,
    pub mode: Mode,
}

impl View {
    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_text_location(&direction),
            EditorCommand::Resize(_) => {}
            EditorCommand::InsertText(character) => self.insert_text_char(character),
            EditorCommand::Backspace => self.delete_backwards(),
            EditorCommand::Delete => self.delete_forwards(),
            EditorCommand::Enter => self.insert_newline(),
            _ => {}
        }
    }

    pub fn change_mode(&mut self, mode: Mode) {
        match &mode {
            Mode::Normal => {
                if let Mode::Insert(_) = self.mode {
                    self.move_text_location(&Direction::Left);
                }
            }
            Mode::Insert(c) => match c {
                'a' => self.move_text_location(&Direction::Right),
                'I' => self.move_text_location(&Direction::Home),
                'A' => self.move_text_location(&Direction::End),
                _ => {}
            },
            _ => {}
        }
        self.mode = mode;
    }

    fn scroll_vertically(&mut self, to: usize) {
        let content_height = self.size.height.saturating_sub(1);
        let offset_changed = if to < self.scroll_offset.row {
            self.scroll_offset.row = to;
            true
        } else if to >= self.scroll_offset.row.saturating_add(content_height) {
            self.scroll_offset.row = to.saturating_sub(content_height).saturating_add(1);
            true
        } else {
            false
        };
        if offset_changed {
            self.mark_redrawn(true);
        }
    }

    fn scroll_horizontally(&mut self, to: usize) {
        let text_col = to.saturating_sub(self.line_padding + 1);
        let visible_width = self.size.width.saturating_sub(self.line_padding + 1);

        let offset_changed = if text_col < self.scroll_offset.col {
            self.scroll_offset.col = text_col;
            true
        } else if to >= self.scroll_offset.col.saturating_add(visible_width) {
            self.scroll_offset.col = text_col.saturating_sub(visible_width);
            true
        } else {
            false
        };
        if offset_changed {
            self.mark_redrawn(true);
        }
    }

    fn scroll_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_to_position();
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(self.scroll_offset)
    }

    fn text_location_to_position(&self) -> Position {
        Position {
            col: self
                .buffer
                .lines
                .get(self.text_location.line_index)
                .map_or(0, |line| {
                    line.width_until(
                        self.text_location
                            .grapheme_index
                            .saturating_sub(self.line_padding + 1),
                    )
                })
                .saturating_add(self.line_padding + 1),
            row: self.text_location.line_index,
        }
    }

    fn move_text_location(&mut self, direction: &Direction) {
        let Size { height, .. } = self.size;
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::Home => self.move_to_start_of_line(),
            Direction::End => self.move_to_end_of_line(),
        }
        self.scroll_location_into_view();
    }

    fn move_up(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    fn move_left(&mut self) {
        if self.text_location.grapheme_index > self.line_padding + 1 {
            self.text_location.grapheme_index -= 1;
        }
    }

    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        if self.text_location.grapheme_index < line_width.saturating_add(self.line_padding) {
            self.text_location.grapheme_index += 1;
        }
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_index = self.line_padding + 1;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count)
            .saturating_add(self.line_padding);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| {
                max(
                    min(
                        line.grapheme_count().saturating_add(self.line_padding),
                        self.text_location.grapheme_index,
                    ),
                    self.line_padding + 1,
                )
            })
    }

    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index = min(self.text_location.line_index, self.buffer.height());
    }

    fn render_line(at: usize, line_text: &str) -> Result<(), Error> {
        Terminal::print_row(at, line_text)
    }

    /// Prints the name and version of the editor in the middle of the terminal
    /// by moving the cursor and printing NAME and VERSION, as above.
    fn draw_welcome_msg(&self, size: Size) {
        let mut welcome_msg;
        let Size { height, width } = size;

        let to_print = format!(">>> {NAME} - v{VERSION}");
        let third_height = height.saturating_div(4);
        let half_width = (width.saturating_sub(to_print.len()).saturating_sub(1)) / 2;

        let len = to_print.len();

        if width == 0 {
            welcome_msg = String::from(" ");
        } else if width <= len {
            welcome_msg = String::from("~");
        } else {
            welcome_msg = format!("~{}{}", " ".repeat(half_width), to_print);
            welcome_msg.truncate(width);
        }

        let result = Terminal::print_row(third_height, &welcome_msg);
        debug_assert!(result.is_ok(), "Failed to render welcome message");
    }

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            total_lines: self.buffer.height(),
            current_line_index: self.text_location.line_index,
            is_modified: self.buffer.dirty,
            filename: format!("{}", self.buffer.file_info),
        }
    }

    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        let buffer = Buffer::load(filename)?;
        self.buffer = buffer;
        self.mark_redrawn(true);
        self.line_padding = self.buffer.height().to_string().len();
        self.text_location = Location {
            grapheme_index: self.line_padding + 1,
            line_index: 0,
        };

        Ok(())
    }

    pub fn save_as(&mut self, filename: &str) -> Result<(), Error> {
        self.buffer.save_as(filename)
    }

    pub fn is_dirty(&self) -> bool {
        self.buffer.is_dirty()
    }

    fn get_adjusted_location(&self) -> Location {
        Location {
            grapheme_index: self
                .text_location
                .grapheme_index
                .saturating_sub(self.line_padding + 1),
            line_index: self.text_location.line_index,
        }
    }

    fn delete_backwards(&mut self) {
        if self.text_location.line_index != 0
            || self
                .text_location
                .grapheme_index
                .saturating_sub(self.line_padding)
                != 0
        {
            if self.text_location.grapheme_index > self.line_padding + 1 {
                self.move_text_location(&Direction::Left);
            } else {
                self.move_text_location(&Direction::Up);
                self.move_text_location(&Direction::End);
            }
            self.delete_forwards();
        }
    }

    fn delete_forwards(&mut self) {
        self.buffer.delete(self.get_adjusted_location());
        self.mark_redrawn(true);
    }

    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.get_adjusted_location());
        self.move_text_location(&Direction::Down);
        self.move_text_location(&Direction::Home);
        self.mark_redrawn(true);
    }

    fn insert_text_char(&mut self, character: char) {
        let old_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);

        self.buffer
            .insert_char(character, self.get_adjusted_location());

        let new_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        let grapheme_delta = new_len.saturating_sub(old_len);
        if grapheme_delta > 0 {
            self.move_text_location(&Direction::Right)
        }
        self.mark_redrawn(true);
    }
}

impl UIComponent for View {
    fn mark_redrawn(&mut self, value: bool) {
        self.needs_redrawn = value;
    }

    fn needs_redrawn(&self) -> bool {
        self.needs_redrawn
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_location_into_view();
    }

    fn draw(&mut self, origin_y: usize) -> Result<(), Error> {
        let Size { height, width } = self.size;
        let end_y = origin_y.saturating_add(height);
        let visible_width = width.saturating_sub(self.line_padding + 1);

        let top = self.scroll_offset.row;
        for current_row in origin_y..end_y {
            let mut default_string;
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                let left_text = self.scroll_offset.col;
                let right_text = left_text.saturating_add(visible_width);
                default_string = format!(
                    "{:width$} ",
                    current_row
                        .saturating_add(self.scroll_offset.row)
                        .saturating_add(1),
                    width = self.line_padding
                )
                .to_owned();
                default_string.push_str(&line.get_visible_graphemes(left_text..right_text));
            } else {
                default_string = format!("{DEFAULT_LINE:width$} ", width = self.line_padding);
                if visible_width > 0 {
                    default_string.push_str(&" ".repeat(visible_width));
                }
            }

            if default_string.len() > width {
                default_string.truncate(width);
            }
            Self::render_line(current_row, &default_string)?;
        }

        if self.buffer.is_empty() {
            self.draw_welcome_msg(self.size);
        }
        self.mark_redrawn(false);

        Ok(())
    }
}

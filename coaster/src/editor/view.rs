mod buffer;
mod commandline;
mod line;

use std::cmp::{max, min};

use self::line::Line;
use super::{
    editorcommand::{Direction, EditorCommand, Mode},
    terminal::{Position, Size, Terminal},
};
use buffer::Buffer;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LINE: &str = "~";

#[derive(Copy, Clone, Default)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

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
    pub fn render(&mut self) {
        if !self.needs_redrawn {
            return;
        }

        let Size { height, width } = self.size;
        if height == 0 || width == 0 {
            return;
        }

        let content_rows = if height == 1 {
            0
        } else {
            height.saturating_sub(1)
        };
        let visible_width = width.saturating_sub(self.line_padding + 1);

        let top = self.scroll_offset.row;
        for current_row in 0..content_rows {
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
            Self::render_line(current_row, &default_string);
        }

        if self.buffer.is_empty() && content_rows > 0 {
            self.draw_welcome_msg(self.size);
        }
        self.draw_status_bar();
        self.needs_redrawn = false;
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::ChangeMode(mode) => {
                self.mode = mode;
                self.needs_redrawn = true;
            }
            EditorCommand::Move(direction) => self.move_text_location(&direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::InsertText(character) => self.insert_text_char(character),
            EditorCommand::Backspace => self.delete_backwards(),
            EditorCommand::Delete => self.delete_forwards(),
            EditorCommand::Enter => self.insert_newline(),
            EditorCommand::InsertCommand(character) => self.insert_command_char(character),
            EditorCommand::ExecuteCommand => self.execute_command(),
        }
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
            self.needs_redrawn = true;
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
            self.needs_redrawn = true;
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
        } else if self.text_location.line_index > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }

    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        if self.text_location.grapheme_index < line_width.saturating_add(self.line_padding + 1) {
            self.text_location.grapheme_index += 1;
        } else {
            self.move_to_start_of_line();
            self.move_down(1);
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
            .saturating_add(self.line_padding + 1);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| {
                max(
                    min(
                        line.grapheme_count().saturating_add(self.line_padding + 1),
                        self.text_location.grapheme_index,
                    ),
                    self.line_padding + 1,
                )
            })
    }

    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index = min(self.text_location.line_index, self.buffer.height());
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
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

    fn draw_status_bar(&self) {
        let status_row = self.size.height.saturating_sub(1);
        let status_text = match self.mode {
            Mode::Insert => "-- INSERT --".to_string(),
            Mode::Normal => "-- NORMAL --".to_string(),
            Mode::Command => self.buffer.command_line.get_display_command(),
            _ => "".to_string(),
        };
        let status_text = if status_text.len() < self.size.width {
            let mut s = status_text;
            s.push_str(&" ".repeat(self.size.width - s.len()));
            s
        } else {
            let mut s = status_text;
            s.truncate(self.size.width);
            s
        };
        let _ = Terminal::print_row(status_row, &status_text);
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.needs_redrawn = true;
            self.line_padding = self.buffer.height().to_string().len();
            self.text_location = Location {
                grapheme_index: self.line_padding + 1,
                line_index: 0,
            };
        }
    }

    fn resize(&mut self, to: Size) {
        self.size = to;
        self.scroll_location_into_view();
        self.needs_redrawn = true;
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
                .saturating_sub(self.line_padding + 1)
                != 0
        {
            self.move_text_location(&Direction::Left);
            self.delete_forwards()
        }
    }

    fn delete_forwards(&mut self) {
        self.buffer.delete(self.get_adjusted_location());
        self.needs_redrawn = true;
    }

    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.get_adjusted_location());
        self.move_text_location(&Direction::Right);
        self.needs_redrawn = true;
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
        self.needs_redrawn = true;
    }

    fn insert_command_char(&mut self, character: char) {
        self.buffer.command_line.insert_char(character);
        self.needs_redrawn = true;
    }

    fn execute_command(&mut self) {
        let command = self.buffer.command_line.as_str();

        for c in command.chars() {
            match c {
                'w' => {
                    let _ = self.buffer.save();
                    self.mode = Mode::Normal;
                }
                'q' => {
                    self.mode = Mode::Exiting;
                    break;
                }
                _ => {}
            }
        }

        self.buffer.command_line.clear();
        self.needs_redrawn = true;
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redrawn: true,
            size: Terminal::size().unwrap_or_default(),
            line_padding: 1,
            text_location: Location {
                grapheme_index: 2,
                line_index: 0,
            },
            scroll_offset: Position::default(),
            mode: Mode::Normal,
        }
    }
}

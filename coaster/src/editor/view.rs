mod buffer;
mod line;
mod location;
mod searchdata;

use std::{
    cmp::{max, min},
    io::Error,
};

use self::line::Line;
use super::{
    DocumentStatus, NAME, VERSION,
    editorcommand::{Direction, EditorCommand, Mode},
    terminal::{Col, Position, Row, Size, Terminal},
    uicomponent::UIComponent,
};
use buffer::Buffer;
pub use location::Location;
use searchdata::SearchData;

const DEFAULT_LINE: &str = "~";

#[derive(PartialEq)]
pub enum SearchDirection {
    NEXT,
    PREV,
    // unused:
    // FIRST,
    // LAST,
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
    pub search_data: SearchData,
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

    pub fn change_mode(&mut self, to: Mode) {
        if self.mode == to {
            return;
        } else if self.mode == Mode::Normal {
            self.search_data = SearchData::default()
        }

        match &to {
            Mode::Normal => match self.mode {
                Mode::Insert(_) => self.move_text_location(&Direction::Left),
                _ => {}
            },
            Mode::Insert(c) => match c {
                'a' => self.move_text_location(&Direction::Right),
                'I' => self.move_text_location(&Direction::Home),
                'A' => self.move_text_location(&Direction::End),
                _ => {}
            },
            _ => {}
        }
        self.mode = to;
    }

    pub fn search(&mut self, query: &str) {
        if query.is_empty() {
            return;
        }

        self.search_data.search_string = String::from(query);
        self.search_data.occurrence_list = self.buffer.search_occurrences(query);
        self.search_data.count = self.search_data.occurrence_list.len();
        self.jump_to_occurrence(SearchDirection::NEXT);
    }

    pub fn jump_to_occurrence(&mut self, direction: SearchDirection) {
        let mut found_occurrence = Location::default();
        let occurrence_list: Vec<Location> = self.search_data.occurrence_list.clone();

        match direction {
            // unused:
            // SearchDirection::FIRST => {
            //     self.search_data.current_occurrence = 1;
            //     found_occurrence = self.search_data.occurrence_list[0].clone();
            // }
            // SearchDirection::LAST => {
            //     self.search_data.current_occurrence = count;
            //     found_occurrence = self.search_data.occurrence_list[count - 1].clone();
            // }
            SearchDirection::NEXT => {
                let mut found = false;
                let mut i = 0;
                while !found && i < occurrence_list.len() {
                    let occurrence = occurrence_list[i];
                    if occurrence.line_idx > self.text_location.line_idx
                        || (occurrence.line_idx == self.text_location.line_idx
                            && occurrence.grapheme_idx > self.text_location.grapheme_idx)
                    {
                        found = true;
                        self.search_data.current_occurrence = i + 1;
                        found_occurrence = occurrence.clone();
                    }
                    i += 1;
                }

                // if no occurrence was found, wrap around to the first / last occurrence
                if !found {
                    self.search_data.current_occurrence = 1;
                    found_occurrence = self.search_data.occurrence_list[0].clone();
                }
            }

            SearchDirection::PREV => {
                let mut found = false;
                let mut i = self.search_data.count - 1;

                while !found && i > 0 {
                    let occurrence = occurrence_list[i];
                    if occurrence.line_idx < self.text_location.line_idx
                        || (occurrence.line_idx == self.text_location.line_idx
                            && occurrence.grapheme_idx < self.text_location.grapheme_idx)
                    {
                        found = true;
                        self.search_data.current_occurrence = i;
                        found_occurrence = self.search_data.occurrence_list
                            [self.search_data.current_occurrence - 1]
                            .clone();
                    }
                    i -= 1;
                }

                // if no occurrence was found, wrap around to the first / last occurrence
                if !found {
                    self.search_data.current_occurrence = self.search_data.count;
                    found_occurrence =
                        self.search_data.occurrence_list[self.search_data.count - 1].clone();
                }
            }
        }

        self.text_location.line_idx = found_occurrence.line_idx;
        self.text_location.grapheme_idx = found_occurrence
            .grapheme_idx
            .saturating_add(self.line_padding + 1);
        self.center_text_location();
    }

    pub fn search_locations_to_positions(&self) -> Vec<Position> {
        self.search_data
            .occurrence_list
            .iter()
            .map(|occurrence| self.location_to_position(occurrence.clone()))
            .collect()
    }

    pub fn get_search_message(&self) -> String {
        format!(
            "{} [{}/{}]",
            self.search_data.search_string,
            self.search_data.current_occurrence,
            self.search_data.count
        )
    }

    fn scroll_vertically(&mut self, to: Row) {
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

    fn scroll_horizontally(&mut self, to: Col) {
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

    fn center_text_location(&mut self) {
        let Size { height, width } = self.size;
        let Position { row, col } = self.location_to_position(self.text_location);
        let vertical_mid = height.div_ceil(2);
        let horizontal_mid = width.div_ceil(2);
        self.scroll_offset.row = row.saturating_sub(vertical_mid);
        self.scroll_offset.col = col.saturating_sub(horizontal_mid);
        self.mark_redrawn(true);
    }

    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.location_to_position(self.text_location);
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    pub fn caret_position(&self) -> Position {
        self.location_to_position(self.text_location)
            .saturating_sub(self.scroll_offset)
    }

    fn location_to_position(&self, location: Location) -> Position {
        let row = location.line_idx;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            line.width_until(
                location.grapheme_idx,
                max(
                    0,
                    location.grapheme_idx.saturating_sub(line.grapheme_count()),
                ),
            )
        });
        Position { col, row }
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
        self.scroll_text_location_into_view();
    }

    fn move_up(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_add(step);
        self.snap_to_valid_line();
        self.snap_to_valid_grapheme();
    }

    fn move_left(&mut self) {
        if self.text_location.grapheme_idx > self.line_padding + 1 {
            self.text_location.grapheme_idx -= 1;
        }
    }

    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
            .map_or(0, Line::grapheme_count);
        if self.text_location.grapheme_idx < line_width.saturating_add(self.line_padding) {
            self.text_location.grapheme_idx += 1;
        }
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_idx = self.line_padding + 1;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_idx = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
            .map_or(0, Line::grapheme_count)
            .saturating_add(self.line_padding);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_idx =
            self.buffer
                .lines
                .get(self.text_location.line_idx)
                .map_or(0, |line| {
                    max(
                        min(
                            line.grapheme_count().saturating_add(self.line_padding),
                            self.text_location.grapheme_idx,
                        ),
                        self.line_padding + 1,
                    )
                })
    }

    fn snap_to_valid_line(&mut self) {
        self.text_location.line_idx = min(
            self.text_location.line_idx,
            self.buffer.height().saturating_sub(1),
        );
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
            current_line_idx: self.text_location.line_idx,
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
            grapheme_idx: self.line_padding + 1,
            line_idx: 0,
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
            grapheme_idx: self
                .text_location
                .grapheme_idx
                .saturating_sub(self.line_padding + 1),
            line_idx: self.text_location.line_idx,
        }
    }

    fn delete_backwards(&mut self) {
        if self.text_location.line_idx != 0
            || self
                .text_location
                .grapheme_idx
                .saturating_sub(self.line_padding)
                != 0
        {
            if self.text_location.grapheme_idx > self.line_padding + 1 {
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
            .get(self.text_location.line_idx)
            .map_or(0, Line::grapheme_count);

        self.buffer
            .insert_char(character, self.get_adjusted_location());

        let new_len = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
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
        self.scroll_text_location_into_view();
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

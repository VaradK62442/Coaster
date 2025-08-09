mod buffer;
mod location;
mod line;

use std::cmp::max;

use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Terminal, Size},
};
use buffer::Buffer;
use location::Location;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LINE: &str = "~";

pub struct View {
    buffer: Buffer,
    pub needs_redrawn: bool,
    size: Size,
    line_padding: usize,
    location: Location,
    scroll_offset: Location,
}

impl View {
    pub fn render(&mut self) {
        if !self.needs_redrawn {
            return;
        }

        let Size {height, width} = self.size;
        if height == 0 || width == 0 {
            return;
        }

        let top = self.scroll_offset.y;
        for current_row in 0..height {
            let mut default_string;
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                let left = self.scroll_offset.x;
                let right = self.scroll_offset.x.saturating_add(width);
                default_string = format!(
                    "{:width$} ",
                    current_row.saturating_add(self.scroll_offset.y).saturating_add(1),
                    width=self.line_padding
                ).to_owned();
                default_string.push_str(&line.get(left..right));
            } else {
                default_string = format!("{DEFAULT_LINE:width$} ", width=self.line_padding);
            }
            Self::render_line(current_row, &default_string);
        }
        
        if self.buffer.is_empty() {
            self.draw_welcome_msg(self.size);
        }
        
        self.needs_redrawn = false;
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Move(direction) => self.move_text_location(&direction),
            EditorCommand::Quit => {}
        }
    }

    pub fn get_position(&self) -> Position {
        self.location.subtract(&self.scroll_offset).into()
    }

    fn move_text_location(&mut self, direction: &Direction) {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = self.size;
        match direction {
            Direction::Up => {
                y = y.saturating_sub(1);
            }
            Direction::Down => {
                y = y.saturating_add(1);
            }
            Direction::Left => {
                x = max(self.line_padding + 1, x.saturating_sub(1));
            }
            Direction::Right => {
                x = x.saturating_add(1);
            }
            Direction::PageUp => {
                y = 0;
            }
            Direction::PageDown => {
                y = height.saturating_sub(1);
            }
            Direction::Home => {
                x = self.line_padding + 1;
            }
            Direction::End => {
                x = width.saturating_sub(1);
            }
        }
        self.location = Location { x, y };
        self.scroll_location_into_view();
    }

    fn scroll_location_into_view(&mut self) {
        let Location { x, y } = self.location;
        let Size { height, width } = self.size;
        let mut offset_changed = false;

        // Scroll vertically
        if y < self.scroll_offset.y {
            self.scroll_offset.y = y;
            offset_changed = true;
        } else if y >= self.scroll_offset.y.saturating_add(height) {
            self.scroll_offset.y = y.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        // Scroll horizontally
        if x < self.scroll_offset.x.saturating_add(self.line_padding).saturating_add(1) {
            self.scroll_offset.x = x.saturating_sub(self.line_padding).saturating_sub(1);
            offset_changed = true;
        } else if x >= self.scroll_offset.x.saturating_add(width) {
            self.scroll_offset.x = x.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }

        self.needs_redrawn = offset_changed;
    }
    
    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }
    
    /// Prints the name and version of the editor in the middle of the terminal
    /// by moving the cursor and printing NAME and VERSION, as above.
    fn draw_welcome_msg(&self, size: Size) {
        let mut welcome_msg;
        let Size {height, width} = size;
        
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

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.needs_redrawn = true;
            self.line_padding = self.buffer.line_number().to_string().len();
            self.location = Location { x: self.line_padding + 1, y: 0 };
        }
    }

    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.scroll_location_into_view();
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
            location: Location { x: 2, y: 0 },
            scroll_offset: Location::default()
        }
    }
}
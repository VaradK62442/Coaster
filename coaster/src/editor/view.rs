mod buffer;

use std::io::Error;
use super::terminal::{Terminal, Size, Position};
use buffer::Buffer;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    pub needs_redrawn: bool,
    size: Size
}

impl View {
    pub fn render(&mut self) -> Result<(), Error> {
        if !self.needs_redrawn {
            return Ok(());
        }

        let Size {height, width} = self.size;
        if height == 0 || width == 0 {
            return Ok(());
        }

        for current_row in 0..height {
            let mut default_string = "~ ".to_owned();
            if let Some(line) = self.buffer.lines.get(current_row) {
                let truncated_line = if line.len() >= width.saturating_sub(2) {
                    &line[0..width]
                } else {
                    line
                };
                default_string.push_str(truncated_line);
            }
            Self::render_line(current_row, &default_string)?;
        }
        
        if self.buffer.is_empty() {
            self.draw_welcome_msg(self.size)?;
        }
        
        self.needs_redrawn = false;
        
        Ok(())
    }
    
    fn render_line(at: usize, line_text: &str) -> Result<(), Error> {
        Terminal::move_caret_to(Position {col: 0, row: at})?;
        Terminal::clear_line()?;
        Terminal::print(line_text)?;

        Ok(())
    }
    
    /// Prints the name and version of the editor in the middle of the terminal
    /// by moving the cursor and printing NAME and VERSION, as above.
    fn draw_welcome_msg(&self, size: Size) -> Result<(), Error> {
        let mut welcome_msg;
        let Size {height, width} = size;
        
        let to_print = format!(">>> {NAME} - v{VERSION}");
        let third_height = height.saturating_div(4);
        let half_width = (width.saturating_sub(to_print.len())) / 2;

        let len = to_print.len();
        
        if width == 0 {
            welcome_msg = " ".to_string();
        } else if width <= len {
            welcome_msg = "~".to_string();
        } else {
            welcome_msg = to_print;
            welcome_msg.truncate(width);
        }

        Terminal::move_caret_to(Position {col: half_width, row: third_height})?;
        Terminal::print(&welcome_msg)?;

        Ok(())
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.needs_redrawn = true;
        }
    }

    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.needs_redrawn = true;
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redrawn: true,
            size: Terminal::size().unwrap_or_default()
        }
    }
}
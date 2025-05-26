mod buffer;

use super::terminal::{Terminal, Size};
use buffer::Buffer;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LINE: &str = "~";
const LINE_PADDING: usize = 3;

pub struct View {
    buffer: Buffer,
    pub needs_redrawn: bool,
    size: Size
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

        for current_row in 0..height {
            let mut default_string;
            if let Some(line) = self.buffer.lines.get(current_row) {
                default_string = format!("{:width$} ", current_row.saturating_add(1), width=LINE_PADDING).to_owned();
                let truncated_line = if line.len() >= width.saturating_sub(2) {
                    &line[0..width]
                } else {
                    line
                };
                default_string.push_str(truncated_line);
            } else {
                default_string = format!("{:width$} ", DEFAULT_LINE, width=LINE_PADDING);
            }
            Self::render_line(current_row, &default_string);
        }
        
        if self.buffer.is_empty() {
            self.draw_welcome_msg(self.size);
        }
        
        self.needs_redrawn = false;
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
            welcome_msg = " ".to_string();
        } else if width <= len {
            welcome_msg = "~".to_string();
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
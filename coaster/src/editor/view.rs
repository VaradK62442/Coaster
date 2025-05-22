use std::io::Error;
use super::terminal::{Terminal, Size, Position};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View;

impl View {
    pub fn render() -> Result<(), Error> {
        let Size {height, ..} = Terminal::size()?;
        for current_row in 0..height {
            Terminal::clear_line()?;
            Terminal::print("~")?;
            if current_row == 0 {
                Terminal::print("Hello, World!")?;
            }
            if current_row.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
        }

        View::draw_welcome_msg()?;

        Ok(())
    }
    
    /// Prints the name and version of the editor in the middle of the terminal
    /// by moving the cursor and printing NAME and VERSION, as above.
    fn draw_welcome_msg() -> Result<(), Error> {
        let Size {height, width} = Terminal::size()?;
        let to_print = format!(">>> {NAME} - v{VERSION}");
        let third_height = height.saturating_div(4);
        let half_width = (width.saturating_sub(to_print.len())) / 2;

        Terminal::move_caret_to(Position {col: half_width, row: third_height})?;
        Terminal::print(&to_print)?;

        Ok(())
    }
}
use crossterm::event::{
    read,
    Event,
    Event::Key,
    KeyCode::Char,
    KeyEvent,
    KeyModifiers
};
mod terminal;
use terminal::{Terminal, Size, Position};
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub const fn default() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }

            let event = read()?;
            self.evaluate_event(&event);
        }

        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => ()
            }
        }
    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print(">>> Exiting\r\n")?;
        } else {
            Self::draw_rows()?;
            Self::draw_welcome_msg()?;
            Terminal::move_cursor_to(Position {x: 0, y: 0})?;
        }
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    fn draw_rows() -> Result<(), Error> {
        let Size {height, ..} = Terminal::size()?;
        for current_row in 0..height {
            Terminal::clear_line()?;
            Terminal::print("~")?;
            if current_row.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
        }

        Ok(())
    }

    /// Prints the name and version of the editor in the middle of the terminal
    /// by moving the cursor and printing NAME and VERSION, as above.
    fn draw_welcome_msg() -> Result<(), Error> {
        let Size {height, width} = Terminal::size()?;
        let to_print = format!(">>> {NAME} - v{VERSION}");
        let third_height = height.saturating_div(4);
        let half_width = (width.saturating_sub(to_print.len())) / 2;

        Terminal::move_cursor_to(Position {x: half_width, y: third_height})?;
        Terminal::print(&to_print)?;

        Ok(())
    }
}
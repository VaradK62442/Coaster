use crossterm::event::{
    read,
    Event,
    Event::Key,
    KeyCode,
    KeyCode::Char,
    KeyEvent,
    KeyModifiers
};
mod terminal;
use terminal::{Terminal, Size, Position};
use std::io::Error;
use core::cmp::min;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Copy, Clone, Default)]
pub struct Location {
    x: usize,
    y: usize,
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    location: Location,
}

impl Editor {
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
            self.evaluate_event(&event)?;
        }

        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), Error> {
        let Size {height, width} = Terminal::size()?;
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                Char('h') | KeyCode::Left => {
                    self.location.x = self.location.x.saturating_sub(1);
                }
                Char('l') | KeyCode::Right => {
                    self.location.x = min(width.saturating_sub(1), self.location.x.saturating_add(1));
                }
                Char('k') | KeyCode::Up => {
                    self.location.y = self.location.y.saturating_sub(1);
                }
                Char('j') | KeyCode::Down => {
                    self.location.y = min(height.saturating_sub(1), self.location.y.saturating_add(1));
                }
                KeyCode::Home => {self.location.x = 0;}
                KeyCode::End => {self.location.x = width.saturating_sub(1);}
                KeyCode::PageUp => {self.location.y = 0;}
                KeyCode::PageDown => {self.location.y = height.saturating_sub(1);}
                _ => ()
            }
        }
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_caret()?;
        Terminal::move_caret_to(Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
        } else {
            Self::draw_rows()?;
            Self::draw_welcome_msg()?;
            Terminal::move_caret_to(Position {col: self.location.x, row: self.location.y})?;
        }
        Terminal::show_caret()?;
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

        Terminal::move_caret_to(Position {col: half_width, row: third_height})?;
        Terminal::print(&to_print)?;

        Ok(())
    }
}
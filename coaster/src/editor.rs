mod terminal;
mod view;

use crossterm::event::{
    read,
    Event,
    Event::Key,
    KeyCode,
    KeyCode::Char,
    KeyEvent,
    KeyModifiers
};
use terminal::{Terminal, Size, Position};
use view::View;
use std::io::Error;
use core::cmp::min;

#[derive(Copy, Clone, Default)]
pub struct Location {
    x: usize,
    y: usize,
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    location: Location,
    view: View
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
            self.view.render()?;
            Terminal::move_caret_to(Position {col: self.location.x, row: self.location.y})?;
        }
        Terminal::show_caret()?;
        Terminal::execute()?;
        Ok(())
    }
}
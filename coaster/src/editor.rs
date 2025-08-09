mod terminal;
mod view;

use crossterm::event::{
    read,
    Event,
    KeyCode::{self, Char},
    KeyEvent,
    KeyEventKind,
    KeyModifiers
};
use terminal::{Terminal, Size, Position};
use view::View;
use std::{
    env,
    io::Error,
    panic::{
        set_hook,
        take_hook
    }
};
use core::cmp::{
    min, max
};

#[derive(Copy, Default, Clone)]
pub struct Location {
    x: usize,
    y: usize,
}

pub struct Editor {
    should_quit: bool,
    location: Location,
    pub view: View
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        let mut view = View::default();
        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            view.load(file_name);
        }
        Ok(Self {
            should_quit: false,
            location: Location { x: view.line_padding + 1, y: 0},
            view
        })
    }
    
    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }

            match read() {
                Ok(event) => self.evaluate_event(event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}");
                    }
                }
            }
        }
    }

    fn evaluate_event(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                code, kind: KeyEventKind::Press, modifiers, ..
            }) => {
                match (code, modifiers) {
                    (Char('q'), KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                    }
                    (Char('h') | KeyCode::Left 
                    | Char('l') | KeyCode::Right
                    | Char('k') | KeyCode::Up
                    | Char('j') | KeyCode::Down
                    | KeyCode::Home | KeyCode::End
                    | KeyCode::PageUp | KeyCode::PageDown, _) => {
                        self.move_point(code);
                    }
                    _ => {}
                }
            }
            Event::Resize(width_u16, height_u16) => {
                let height = height_u16 as usize;
                let width = width_u16 as usize;
                self.view.resize(Size {
                    height, width
                });
            }
            _ => {}
        }
    }

    fn move_point(&mut self, code: KeyCode) {
        let Size {height, width} = Terminal::size().unwrap_or_default();
        match code {
            Char('h') | KeyCode::Left => {
                self.location.x = max(self.view.line_padding + 1, self.location.x.saturating_sub(1));
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
            KeyCode::Home => {self.location.x = self.view.line_padding + 1;}
            KeyCode::End => {self.location.x = width.saturating_sub(1);}
            KeyCode::PageUp => {self.location.y = 0;}
            KeyCode::PageDown => {self.location.y = height.saturating_sub(1);}
            _ => ()
        }
    }    

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        self.view.render();
        let _ = Terminal::move_caret_to(Position {
            col: self.location.x,
            row: self.location.y
        });
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print(">>> Exiting.\r\n");
        }
    }
}
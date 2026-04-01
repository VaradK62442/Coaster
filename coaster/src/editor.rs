mod documentstatus;
mod editorcommand;
mod fileinfo;
mod messagebar;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;

use self::{messagebar::MessageBar, terminal::Size};
use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use documentstatus::DocumentStatus;
use editorcommand::{EditorCommand, Mode};
use statusbar::StatusBar;
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};
use terminal::Terminal;
use uicomponent::UIComponent;
use view::View;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMMAND_PREFIX: &str = ":";

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    pub view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    terminal_size: Size,
    title: String,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();
        editor.resize(size);

        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            editor.view.load(file_name);
        }

        editor.message_bar.update_message(editor.get_status_text());
        editor.refresh_status();
        Ok(editor)
    }

    fn get_status_text(&self) -> String {
        return match self.view.mode {
            Mode::Insert => "-- INSERT --".to_string(),
            Mode::Normal => "-- NORMAL --".to_string(),
            Mode::Exiting => ">>> Exiting...".to_string(),
            Mode::Command => COMMAND_PREFIX.to_string(),
        };
    }

    pub fn resize(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });
        self.message_bar.resize(Size {
            height: 1,
            width: size.width,
        });
        self.status_bar.resize(Size {
            height: 1,
            width: size.width,
        })
    }

    pub fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.filename);
        self.status_bar.update_status(status);
        self.message_bar.update_message(self.get_status_text());

        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
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

            let status = self.view.get_status();
            self.status_bar.update_status(status);
        }
    }

    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            if let Ok(command) = EditorCommand::try_from((event, self.view.mode.clone())) {
                if let EditorCommand::Resize(size) = command {
                    self.resize(size);
                } else if let EditorCommand::ChangeMode(mode) = command {
                    self.view.change_mode(mode);
                    self.message_bar.update_message(self.get_status_text());
                } else if let EditorCommand::InsertCommand(c) = command {
                    self.message_bar.insert_command_char(c);
                } else if let EditorCommand::ExecuteCommand = command {
                    self.execute_command();
                } else {
                    self.view.handle_command(command);
                }

                match self.view.mode {
                    Mode::Exiting => self.should_quit = true,
                    _ => {}
                }
            }
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Received and discarded unsupported or non-press event.");
            }
        }
    }

    fn execute_command(&mut self) {
        let command = self.message_bar.get_current_message();
        let mut new_mode = Mode::Normal;

        for c in command.chars() {
            match c {
                'w' => {
                    let _ = self.view.save();
                }
                'q' => {
                    new_mode = Mode::Exiting;
                    break;
                }
                _ => {}
            }
        }

        self.view.change_mode(new_mode);
        self.message_bar.update_message(self.get_status_text());
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }
        let _ = Terminal::hide_caret();

        self.message_bar
            .render(self.terminal_size.height.saturating_sub(1));
        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        }
        if self.terminal_size.height > 2 {
            self.view.render(0);
        }

        let _ = Terminal::move_caret_to(self.view.caret_position());
        let _ = Terminal::set_caret_style(self.view.mode.clone());
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

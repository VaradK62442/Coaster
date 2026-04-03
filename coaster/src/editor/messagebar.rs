use std::io::Error;

use super::{
    terminal::{Size, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct MessageBar {
    current_message: String,
    needs_redrawn: bool,
    showing_error: bool,
}

impl MessageBar {
    pub fn update_message(&mut self, new_msg: &str) {
        if new_msg != self.current_message {
            self.current_message = new_msg.to_string();
            self.mark_redrawn(true);
            self.showing_error = false;
        }
    }

    pub fn insert_command_char(&mut self, command: char) {
        self.current_message.push(command);
        self.mark_redrawn(true);
    }

    pub fn delete_last_char(&mut self) {
        if self.current_message.len() > 1 {
            self.current_message.pop();
            self.mark_redrawn(true);
        }
    }

    pub fn get_current_message(&self) -> &str {
        &self.current_message
    }

    pub fn show_error(&mut self, error: &str) {
        self.update_message(error);
        self.showing_error = true;
    }

    pub fn is_showing_error(&self) -> bool {
        self.showing_error
    }
}

impl UIComponent for MessageBar {
    fn mark_redrawn(&mut self, value: bool) {
        self.needs_redrawn = value;
    }

    fn needs_redrawn(&self) -> bool {
        self.needs_redrawn
    }

    fn set_size(&mut self, _: Size) {}

    fn draw(&mut self, origin: usize) -> Result<(), Error> {
        Terminal::print_row(origin, &self.current_message)
    }
}

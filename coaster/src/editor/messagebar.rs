use std::io::Error;

use super::{
    terminal::{Size, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct MessageBar {
    current_message: String,
    needs_redrawn: bool,
}

impl MessageBar {
    pub fn update_message(&mut self, new_msg: String) {
        if new_msg != self.current_message {
            self.current_message = new_msg;
            self.mark_redrawn(true);
        }
    }

    pub fn insert_command_char(&mut self, command: char) {
        self.current_message.push(command);
        self.mark_redrawn(true);
    }

    pub fn get_current_message(&self) -> &str {
        &self.current_message
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

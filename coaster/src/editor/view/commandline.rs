const COMMAND_PREFIX: &str = ":";

#[derive(Default)]
pub struct CommandLine {
    command: String,
}

impl CommandLine {
    pub fn insert_char(&mut self, character: char) {
        self.command.push(character);
    }

    pub fn clear(&mut self) {
        self.command.clear();
    }

    pub fn get_display_command(&self) -> String {
        let mut full = String::with_capacity(COMMAND_PREFIX.len() + self.command.len());
        full.push_str(COMMAND_PREFIX);
        full.push_str(&self.command);
        full
    }

    pub fn as_str(&self) -> &str {
        &self.command
    }
}

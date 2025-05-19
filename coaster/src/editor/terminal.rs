use crossterm::cursor::{
    MoveTo,
    Hide,
    Show
};
use crossterm::{queue, Command};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode,
    enable_raw_mode,
    size,
    Clear,
    ClearType
};
use std::io::{stdout, Error, Write};
use core::fmt::Display;


#[derive(Copy, Clone)]
pub struct Size {
    pub height: usize,
    pub width: usize
}

#[derive(Copy, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

/// Represents the Terminal.
pub struct Terminal;

impl Terminal {
    pub fn terminate() -> Result<(), Error> {
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }
    
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        queue!(stdout(), Hide)?;
        Self::clear_screen()?;
        queue!(stdout(), Show)?;
        Self::move_cursor_to(Position {x: 0, y: 0})?;
        Self::execute()?;
        Ok(())
    }

    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    /// Moves cursor to given Position
    /// ### Arguments
    /// * `position` - the `Position` to move the cursor to.
    pub fn move_cursor_to(position: Position) -> Result<(), Error> {
        Self::queue_command(MoveTo(position.x as u16, position.y as u16))?;
        Ok(())
    }

    pub fn hide_cursor() -> Result<(), Error> {
        Self::queue_command(Hide)?;
        Ok(())
    }

    pub fn show_cursor() -> Result<(), Error> {
        Self::queue_command(Show)?;
        Ok(())
    }

    pub fn print<T: Display>(string: T) -> Result<(), Error> {
        Self::queue_command(Print(string))?;
        Ok(())
    }

    /// Returns the current size of the terminal.
    /// * A `Size` representing the terminal size.
    pub fn size() -> Result<Size, Error> {
        let (width, height) = size()?;
        let width = width as usize;
        let height = height as usize;
        Ok(Size { height, width })
    }

    /// Flushes the output stream so that any bufferred contents reach their destination.
    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }

    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }
}
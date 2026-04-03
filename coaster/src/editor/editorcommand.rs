use crossterm::event::{
    Event,
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};
use std::convert::TryFrom;

use super::terminal::Size;

#[derive(Clone, Copy)]
pub enum Direction {
    PageUp,
    PageDown,
    Home,
    End,
    Up,
    Left,
    Right,
    Down,
}

#[derive(Clone, Copy)]
pub enum EditorCommand {
    ChangeMode(Mode),
    Move(Direction),
    Resize(Size),
    InsertText(char),
    Backspace,
    Delete,
    Enter,
    InsertCommand(char),
    DeleteCommand,
    ExecuteCommand,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}

#[allow(clippy::as_conversions)]
impl TryFrom<(Event, Mode)> for EditorCommand {
    type Error = String;
    fn try_from((event, mode): (Event, Mode)) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (mode, code, modifiers) {
                // commands
                (Mode::Command, KeyCode::Char(character), KeyModifiers::NONE) => {
                    Ok(Self::InsertCommand(character))
                }
                (Mode::Command, KeyCode::Enter, KeyModifiers::NONE) => Ok(Self::ExecuteCommand),
                (Mode::Command, KeyCode::Backspace, _) => Ok(Self::DeleteCommand),

                // changing mode
                (Mode::Normal, KeyCode::Char('i'), KeyModifiers::NONE) => {
                    Ok(Self::ChangeMode(Mode::Insert))
                }
                (Mode::Normal, KeyCode::Char(':'), _) => Ok(Self::ChangeMode(Mode::Command)),
                (_, KeyCode::Esc, _) => Ok(Self::ChangeMode(Mode::Normal)),

                // editing
                (
                    Mode::Insert,
                    KeyCode::Char(character),
                    KeyModifiers::NONE | KeyModifiers::SHIFT,
                ) => Ok(Self::InsertText(character)),
                (Mode::Insert, KeyCode::Backspace, _) => Ok(Self::Backspace),
                (Mode::Insert, KeyCode::Delete, _) => Ok(Self::Delete),
                (Mode::Insert, KeyCode::Tab, _) => Ok(Self::InsertText('\t')),
                (Mode::Insert, KeyCode::Enter, _) => Ok(Self::Enter),

                // navigation
                (_, KeyCode::PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (_, KeyCode::PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (_, KeyCode::Home, _) => Ok(Self::Move(Direction::Home)),
                (_, KeyCode::End, _) => Ok(Self::Move(Direction::End)),
                (_, KeyCode::Left, _) => Ok(Self::Move(Direction::Left)),
                (_, KeyCode::Down, _) => Ok(Self::Move(Direction::Down)),
                (_, KeyCode::Up, _) => Ok(Self::Move(Direction::Up)),
                (_, KeyCode::Right, _) => Ok(Self::Move(Direction::Right)),
                (Mode::Normal, Char('h'), KeyModifiers::NONE) => Ok(Self::Move(Direction::Left)),
                (Mode::Normal, Char('j'), KeyModifiers::NONE) => Ok(Self::Move(Direction::Down)),
                (Mode::Normal, Char('k'), KeyModifiers::NONE) => Ok(Self::Move(Direction::Up)),
                (Mode::Normal, Char('l'), KeyModifiers::NONE) => Ok(Self::Move(Direction::Right)),

                _ => Err(format!("Unsupported key code: {code:?}")),
            },
            Event::Resize(width_u16, height_u16) => Ok(Self::Resize(Size {
                height: height_u16 as usize,
                width: width_u16 as usize,
            })),
            _ => Err(format!("Unsupported event: {event:?}")),
        }
    }
}

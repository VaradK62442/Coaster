use crossterm::event::{
    Event,
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};
use std::convert::TryFrom;

use super::super::editor::{COMMAND_PREFIX, SEARCH_PREFIX};
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
    // commands
    InsertCommand(char),
    DeleteCommand,
    ExecuteCommand,

    // mode
    ChangeMode(Mode),

    // inserting
    InsertText(char),
    Backspace,
    Delete,
    Enter,

    // navigation
    Move(Direction),
    Resize(Size),

    // searching
    SearchNext,
    SearchPrevious,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    Normal,
    Insert(char),
    Command,
    Search,
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
                // commands / searching
                (Mode::Command | Mode::Search, KeyCode::Char(character), _) => {
                    Ok(Self::InsertCommand(character))
                }
                (Mode::Command | Mode::Search, KeyCode::Enter, KeyModifiers::NONE) => {
                    Ok(Self::ExecuteCommand)
                }
                (Mode::Command | Mode::Search, KeyCode::Backspace, KeyModifiers::NONE) => {
                    Ok(Self::DeleteCommand)
                }

                // changing mode
                (Mode::Normal, KeyCode::Char('i'), KeyModifiers::NONE) => {
                    Ok(Self::ChangeMode(Mode::Insert('i')))
                }
                (Mode::Normal, KeyCode::Char('a'), KeyModifiers::NONE) => {
                    Ok(Self::ChangeMode(Mode::Insert('a')))
                }
                (Mode::Normal, KeyCode::Char('I'), KeyModifiers::SHIFT) => {
                    Ok(Self::ChangeMode(Mode::Insert('I')))
                }
                (Mode::Normal, KeyCode::Char('A'), KeyModifiers::SHIFT) => {
                    Ok(Self::ChangeMode(Mode::Insert('A')))
                }
                (Mode::Normal, KeyCode::Char(COMMAND_PREFIX), KeyModifiers::NONE) => {
                    Ok(Self::ChangeMode(Mode::Command))
                }
                (Mode::Normal, KeyCode::Char(SEARCH_PREFIX), KeyModifiers::NONE) => {
                    Ok(Self::ChangeMode(Mode::Search))
                }
                (Mode::Normal, KeyCode::Char('n'), KeyModifiers::NONE) => Ok(Self::SearchNext),
                (Mode::Normal, KeyCode::Char('N'), KeyModifiers::SHIFT) => Ok(Self::SearchPrevious),
                (_, KeyCode::Esc, _) => Ok(Self::ChangeMode(Mode::Normal)),

                // inserting
                (
                    Mode::Insert(_),
                    KeyCode::Char(character),
                    KeyModifiers::NONE | KeyModifiers::SHIFT,
                ) => Ok(Self::InsertText(character)),
                (Mode::Insert(_), KeyCode::Backspace, _) => Ok(Self::Backspace),
                (Mode::Insert(_), KeyCode::Delete, _) => Ok(Self::Delete),
                (Mode::Insert(_), KeyCode::Tab, _) => Ok(Self::InsertText('\t')),
                (Mode::Insert(_), KeyCode::Enter, _) => Ok(Self::Enter),

                // editing
                (Mode::Normal, KeyCode::Char('x'), KeyModifiers::NONE) => Ok(Self::Delete),

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
                (Mode::Normal, Char('0'), KeyModifiers::NONE) => Ok(Self::Move(Direction::Home)),
                (Mode::Normal, Char('$'), KeyModifiers::NONE) => Ok(Self::Move(Direction::End)),

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

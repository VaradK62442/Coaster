use crossterm::event::{
    Event,
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};
use std::convert::TryFrom;

use super::terminal::Size;

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
pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Quit,
    Insert(char),
    ChangeMode(Mode),
}

#[derive(Clone)]
pub enum Mode {
    Normal,
    Insert,
}

#[allow(clippy::as_conversions)]
impl TryFrom<(Event, Mode)> for EditorCommand {
    type Error = String;
    fn try_from((event, mode): (Event, Mode)) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (mode, code, modifiers) {
                // quitting
                (_, KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),

                // changing mode
                (Mode::Normal, KeyCode::Char('i'), KeyModifiers::NONE) => {
                    Ok(Self::ChangeMode(Mode::Insert))
                }
                (Mode::Insert, KeyCode::Esc, _) => Ok(Self::ChangeMode(Mode::Normal)),

                // insertion
                (
                    Mode::Insert,
                    KeyCode::Char(character),
                    KeyModifiers::NONE | KeyModifiers::SHIFT,
                ) => Ok(Self::Insert(character)),

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

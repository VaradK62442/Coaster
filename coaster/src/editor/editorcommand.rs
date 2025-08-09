use crossterm::event::{
    Event,
    KeyCode,
    KeyEvent,
    KeyModifiers
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
    Down
}
pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Quit
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (KeyCode::PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (KeyCode::Home, _) => Ok(Self::Move(Direction::Home)),
                (KeyCode::End, _) => Ok(Self::Move(Direction::End)),
                (KeyCode::Up, _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Left, _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Right, _) => Ok(Self::Move(Direction::Right)),
                (KeyCode::Down, _) => Ok(Self::Move(Direction::Down)),
                _ => Err(format!("Unsupported key code: {code:?}"))
            }
            Event::Resize(width_u16, height_u16) => {
                #[allow(clippy::as_conversions)]
                let height = height_u16 as usize;
                #[allow(clippy::as_conversions)]
                let width = width_u16 as usize;

                Ok(Self::Resize(Size { height, width }))
            }
            _ => Err(format!("Unsupported event: {event:?}"))
        }
    }
}
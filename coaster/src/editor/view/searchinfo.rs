use crate::editor::view::{Line, Position};

use super::Location;

pub struct SearchInfo {
    pub prev_location: Location,
    pub prev_scroll_offset: Position,
    pub query: Line,
}

use enum_iterator::Sequence;
use enum_map::Enum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Sequence, Enum, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }
}

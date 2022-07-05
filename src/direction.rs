use enum_map::Enum;
use serde::{Serialize, Deserialize};



#[derive(Debug, Clone, Enum, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}


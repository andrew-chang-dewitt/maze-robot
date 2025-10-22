mod maze;
mod robot;

use std::fmt::Display;

pub use crate::maze::Maze;
pub use crate::robot::Robot;

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::Up => "Up",
            Self::Right => "Right",
            Self::Down => "Down",
            Self::Left => "Left",
        };

        write!(f, "{out}")
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Cell {
    Finish,
    Open,
    Wall,
}

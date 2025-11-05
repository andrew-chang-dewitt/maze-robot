mod maze;
mod robot;
mod solution;

use std::fmt::Display;

pub use crate::maze::{Maze, MazeError};
pub use crate::robot::Robot;
pub use crate::solution::solve;

pub const DIR_ARR: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn reverse(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::North => "North",
            Self::East => "East",
            Self::South => "South",
            Self::West => "West",
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

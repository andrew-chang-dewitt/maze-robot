use std::fmt::Display;

// pub mod dist_maze;
pub mod text_maze;
pub mod traits;

use traits::{MazeError, Robot};

/// utility function for easily getting handle to a new concrete Robot instance from a given Maze-like source
pub fn new_bot<R: Robot, M: TryInto<R, Error = MazeError>>(maze: M) -> Result<R, MazeError> {
    maze.try_into()
}
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Finish,
    Open,
    Wall,
}

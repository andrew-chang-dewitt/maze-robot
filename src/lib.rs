use std::{error::Error, fmt::Display};

pub mod dist_maze;
pub mod text_maze;
pub mod traits;

use traits::{MazeError, Robot};

/// utility function for easily getting handle to a new concrete Robot instance from a given Maze-like source
pub fn new_bot<R: Robot, M: TryInto<R, Error = MazeError>>(maze: M) -> Result<R, MazeError> {
    maze.try_into()
}

/// Values representing cardinal directions
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    /// Get a Direction that is the opposite of this one.
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

/// A utility value used to make a fixed-size iterable of all possible directions
pub const DIR_ARR: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

/// Values to represent parts of a Maze.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    /// The end of a maze.
    Finish,
    /// A normally open cell that is currently occupied by another Robot. Includes a value that
    /// should be able to be used to uniquely identify which Robot it is.
    Occupied(u32),
    /// A cell that is not blocked & is able to be travled into.
    Open,
    /// A cell that is always blocked.
    Wall,
}

impl Cell {
    const FINISH: u8 = 0x00;
    const OCCUPIED: u8 = 0x01;
    const OPEN: u8 = 0x02;
    const WALL: u8 = 0x03;
}

impl TryFrom<[u8; 5]> for Cell {
    type Error = CellDecodeError;

    fn try_from(bytes: [u8; 5]) -> Result<Self, Self::Error> {
        // the first byte indicates what type of Cell it is
        let first = &bytes[0];
        let rest: [u8; 4] = [bytes[1], bytes[2], bytes[3], bytes[4]];

        match first {
            &Self::FINISH => Ok(Cell::Finish),
            &Self::OCCUPIED => {
                let id = u32::from_le_bytes(rest);
                Ok(Cell::Occupied(id))
            }
            &Self::OPEN => Ok(Cell::Open),
            &Self::WALL => Ok(Cell::Wall),
            _ => Err(CellDecodeError::new(bytes)),
        }
    }
}

const NULL_B: u8 = b"\0"[0];

impl Into<[u8; 5]> for Cell {
    fn into(self) -> [u8; 5] {
        match self {
            // Occupied needs id as 4-byte unsigned int at end
            Cell::Occupied(id) => {
                let id_b = id.to_le_bytes();
                [Self::OCCUPIED, id_b[0], id_b[1], id_b[2], id_b[3]]
            }
            // rest just need filled w/ empty values
            Cell::Finish => [Self::FINISH, NULL_B, NULL_B, NULL_B, NULL_B],
            Cell::Open => [Self::OPEN, NULL_B, NULL_B, NULL_B, NULL_B],
            Cell::Wall => [Self::WALL, NULL_B, NULL_B, NULL_B, NULL_B],
        }
    }
}

#[derive(Debug)]
pub struct CellDecodeError {
    bytes: [u8; 5],
    src: Option<Box<dyn Error + Send + Sync>>,
}

impl CellDecodeError {
    pub fn new(bytes: [u8; 5]) -> Self {
        Self { bytes, src: None }
    }

    pub fn caused_by(self, err: impl Error + Send + Sync + 'static) -> Self {
        Self {
            bytes: self.bytes,
            src: Some(Box::new(err)),
        }
    }
}

impl Display for CellDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid Cell bytes: {:?}", self.bytes)
    }
}

impl std::error::Error for CellDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.src {
            Some(e) => Some(&**e),
            _ => None,
        }
    }
}

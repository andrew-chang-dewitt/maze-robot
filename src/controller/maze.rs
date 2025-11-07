use std::{
    error::Error,
    fmt::{Debug, Display},
};

use super::{Cell, Direction};

/// A Maze is the actual environment our robot will move in.
///
/// As a maze is unknown to the robot, it provides very little in the way of information, exposing
/// only two capabilities: look in some direction (`look_dir`) & move in some direction
/// (`move_dir`).
pub trait Maze: Debug + Display {
    /// Look in the given direction tell the caller what type of Cell was seen.
    fn look_dir(&self, direction: Direction) -> Cell;

    /// Attempt to move in the given direction.
    ///
    /// If not possible, a `MazeError::MoveError` will be returned.
    fn move_dir(&mut self, direction: Direction) -> Result<(), MazeError>;
}

#[derive(Debug)]
pub enum MazeError {
    CreationError(String),
    MoveError(Direction, String),
}

impl Display for MazeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::CreationError(msg) => format!("CreationError: {msg}"),
            Self::MoveError(direction, state) => {
                format!("MoveError: unable to go {direction} from current location:\n\n{state}\n")
            }
        };

        write!(f, "MazeError:{out}")
    }
}

impl Error for MazeError {}

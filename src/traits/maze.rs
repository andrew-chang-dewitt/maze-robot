use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{Cell, Direction};

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
pub enum MazeErrorType {
    CreationError(String),
    MoveError(Direction, String),
}

#[derive(Debug)]
pub struct MazeError {
    typ: MazeErrorType,
    src: Option<Box<dyn Error + Send + Sync>>,
}

impl MazeError {
    pub fn new(typ: MazeErrorType) -> Self {
        Self { typ, src: None }
    }

    pub fn caused_by(self, err: impl Error + Send + Sync + 'static) -> Self {
        Self {
            typ: self.typ,
            src: Some(Box::new(err)),
        }
    }

    pub fn get_type(&self) -> &MazeErrorType {
        &self.typ
    }
}

impl Display for MazeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match &self.typ {
            MazeErrorType::CreationError(msg) => format!("CreationError: {msg}"),
            MazeErrorType::MoveError(direction, state) => {
                format!("MoveError: unable to go {direction} from current location:\n\n{state}\n")
            }
        };

        write!(f, "MazeError:{out}")
    }
}

impl Error for MazeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.src {
            Some(e) => Some(&**e),
            _ => None,
        }
    }
}

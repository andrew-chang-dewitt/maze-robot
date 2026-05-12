use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::Direction;

mod maze;
mod multi_maze;
mod multi_maze_handle;
mod robot;

pub use maze::Maze;
pub use multi_maze::MultiMaze;
pub use multi_maze_handle::MultiMazeHandle;
pub use robot::{Robot, RobotInternal};

#[derive(Debug)]
pub enum RobotErrorType {
    /// Robot has no swarm transport attached (call `join_swarm` first).
    NotJoined,
    /// User-provided `TryInto<Vec<u8>>` encoder returned an error.
    EncodeError(String),
    /// User-provided `TryFrom<Vec<u8>>` decoder returned an error.
    DecodeError(String),
    /// Transport-level I/O failure during send or recv.
    TransportError(String),
}

#[derive(Debug)]
pub struct RobotError {
    typ: RobotErrorType,
    src: Option<Box<dyn Error + Send + Sync>>,
}

impl RobotError {
    pub fn new(typ: RobotErrorType) -> Self {
        Self { typ, src: None }
    }

    pub fn caused_by(self, err: impl Error + Send + Sync + 'static) -> Self {
        Self {
            typ: self.typ,
            src: Some(Box::new(err)),
        }
    }

    pub fn get_type(&self) -> &RobotErrorType {
        &self.typ
    }
}

impl Display for RobotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match &self.typ {
            RobotErrorType::NotJoined => "NotJoined: robot has no swarm transport".to_string(),
            RobotErrorType::EncodeError(msg) => format!("EncodeError: {msg}"),
            RobotErrorType::DecodeError(msg) => format!("DecodeError: {msg}"),
            RobotErrorType::TransportError(msg) => format!("TransportError: {msg}"),
        };
        write!(f, "RobotError:{out}")
    }
}

impl Error for RobotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.src {
            Some(e) => Some(&**e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum MazeErrorType {
    CreationError(String),
    MoveError(Direction, String),
    UnknownRobot(usize),
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
            MazeErrorType::UnknownRobot(id) => {
                format!("UnknownRobot: Robot w/ id {id} not known")
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

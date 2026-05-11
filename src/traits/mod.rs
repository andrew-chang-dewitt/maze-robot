use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::Direction;

mod maze;
mod multi_maze;
mod robot;

pub use maze::Maze;
pub use multi_maze::MultiMaze;
pub use robot::{Robot, RobotInternal};

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

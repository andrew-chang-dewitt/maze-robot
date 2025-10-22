use std::{error::Error, fmt::Display};

use crate::{
    Cell, Direction, Maze,
    maze::{MazeError, TextMaze},
};

pub struct Robot<M: Maze> {
    state: M,
}

impl<M: Maze> Robot<M> {
    fn new(state: M) -> Self {
        Robot { state }
    }

    fn peek(&self, direction: Direction) -> Cell {
        self.state.look_dir(direction)
    }
}

impl TryFrom<&str> for Robot<TextMaze> {
    type Error = RobotError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Robot::new(TextMaze::try_from(value).map_err(|e| e.into())?))
    }
}

#[derive(Debug)]
pub enum RobotError {
    CreationError(String),
}

impl Display for RobotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::CreationError(msg) => format!("CreationError: {msg}"),
        };

        write!(f, "MazeError:{out}")
    }
}

impl Error for RobotError {}

impl Into<RobotError> for MazeError {
    fn into(self) -> RobotError {
        match self {
            MazeError::CreationError(msg) => RobotError::CreationError(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;

    struct MockMaze;
    impl Maze for MockMaze {
        fn look_dir(&self, direction: Direction) -> Cell {
            Cell::Wall
        }
    }

    #[fixture]
    fn wall_maze() -> MockMaze {
        MockMaze
    }

    #[rstest]
    fn test_peek(
        #[values(Direction::Up, Direction::Right, Direction::Down, Direction::Left)]
        direction: Direction,
        wall_maze: MockMaze,
    ) {
        let rob = Robot::new(wall_maze);

        match rob.peek(direction) {
            Cell::Wall => (),
            _ => panic!("Expected peeking in {direction:?} to return Cell::Wall"),
        }
    }
}

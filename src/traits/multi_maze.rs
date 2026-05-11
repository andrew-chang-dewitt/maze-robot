use std::fmt::{Debug, Display};

use crate::{Cell, Direction};

use super::MazeError;

/// Like [super::maze::`Maze`], but allows for possibility of multiple robots in the same maze.
pub trait MultiMaze: Debug + Display {
    /// Look in the given direction from the robot with the corresponding ID & tell the caller what type of Cell was seen.
    ///
    /// If no known robot matches the given ID, a [`MazeError::UnknownRobot`] will be returned
    fn look_dir(&self, id: usize, direction: Direction) -> Result<Cell, MazeError>;

    /// Attempt to move the robot with the corresponding ID in the given direction.
    ///
    /// If not possible, a [`MazeError::MoveError`] will be returned.
    fn move_dir(&mut self, id: usize, direction: Direction) -> Result<(), MazeError>;
}

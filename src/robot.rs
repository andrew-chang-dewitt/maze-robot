use crate::maze::{Direction, Maze, MazeError, TextMaze};

pub struct Robot<M: Maze> {
    env: M,
    loc: M::Key,
}

impl<M: Maze> Robot<M> {
    fn new(self, env: M) -> Self {
        Self {
            loc: env.get_start(),
            env,
        }
    }

    fn go(&mut self, direction: Direction) -> Result<M::Key, RobotError<M>> {
        self.loc = match self.env.lookup_by_direction(self.loc, direction) {
            Some(key) => Ok(key),
            None => Err(RobotError::NavigationError(self.loc, direction)),
        }?;

        Ok(self.loc)
    }

    fn peek(self, direction: Direction) -> Option<M::Key> {
        self.env.lookup_by_direction(self.loc, direction)
    }
}

pub enum RobotError<M: Maze> {
    UnknownError(String),
    NavigationError(M::Key, Direction),
}

impl<M: Maze> From<MazeError> for RobotError<M> {
    fn from(value: MazeError) -> Self {
        match value {
            _ => Self::UnknownError(String::from(
                "An unknown error in maze navigation or processing occurred.",
            )),
        }
    }
}

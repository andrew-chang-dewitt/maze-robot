use std::{cell::RefCell, fmt::Display};

use super::{Cell, DIR_ARR, Direction, Maze, maze::MazeError};

pub trait Robot {
    fn get_internal(&self) -> &RobotInternal;

    fn peek(&self, direction: Direction) -> Cell {
        self.get_internal().peek(direction)
    }

    fn peek_all(&self) -> [(Cell, Direction); 4] {
        self.get_internal().peek_all()
    }

    fn go(&self, direction: Direction) -> Result<(), MazeError> {
        self.get_internal().go(direction)
    }
}

#[derive(Debug)]
pub struct RobotInternal {
    // maze is actually an _external_ enviroment the robot exists _inside_ of
    // the robot has no notion of state itself--it simply looks & attempts to travel in specified
    // directions. as far as it is concerned, no state changes happen for either of those actions.
    // to model this, the **interior mutability** pattern is used by placing the env inside a
    // RefCell inside our Robot, then calling env methods on it to perform Robot actions w/out
    // worrying about any side effects.
    //
    // additionally, a dynamic trait object is used here since the robot doesn't care what kind of
    // maze environment it is in--it always works the same. this keeps the user of Robot from
    // having to know anything about the Maze construct.
    env: RefCell<Box<dyn Maze>>,
}

impl RobotInternal {
    pub fn new<M: 'static + Maze>(maze: M) -> Self {
        Self {
            env: RefCell::new(Box::new(maze)),
        }
    }

    pub fn peek(&self, direction: Direction) -> Cell {
        self.env.borrow().look_dir(direction)
    }

    pub fn peek_all(&self) -> [(Cell, Direction); 4] {
        DIR_ARR.map(|dir| (self.peek(dir), dir))
    }

    pub fn go(&self, direction: Direction) -> Result<(), MazeError> {
        #[cfg(test)]
        {
            println!("[Robot::go] BEGIN go {direction} from {self}");
        }
        self.env
            .borrow_mut()
            .move_dir(direction)
            .map_err(|e| e.into())
    }
}

impl Display for RobotInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.env.borrow().to_string();

        write!(f, "Robot state:\n{state}")
    }
}

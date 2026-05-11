use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    Cell, Direction,
    traits::{Maze, MazeError, MultiMaze},
};

#[derive(Debug)]
pub struct MultiMazeHandle {
    inner: Rc<RefCell<Box<dyn MultiMaze>>>,
    id: usize,
}

impl MultiMazeHandle {
    pub fn new(inner: Rc<RefCell<Box<dyn MultiMaze>>>, id: usize) -> Self {
        Self { inner, id }
    }
}

impl Maze for MultiMazeHandle {
    fn look_dir(&self, direction: Direction) -> Result<Cell, MazeError> {
        self.inner.borrow().look_dir(self.id, direction)
    }

    fn move_dir(&mut self, direction: Direction) -> Result<(), MazeError> {
        self.inner.borrow_mut().move_dir(self.id, direction)
    }
}

impl Display for MultiMazeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.borrow())
    }
}

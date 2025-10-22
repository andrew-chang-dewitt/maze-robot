mod maze;
mod robot;

pub use crate::maze::Maze;
pub use crate::robot::Robot;

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Cell {
    Finish,
    Open,
    Wall,
}

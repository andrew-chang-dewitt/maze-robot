use std::collections::{HashMap, VecDeque};

use maze_robot::{Direction, Maze, Robot};

type CellGraph = HashMap<usize, [Option<usize>; 4]>;

const DIR_ARR: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

pub fn find_solution<M: Maze>(robot: Robot<M>) -> anyhow::Result<(CellGraph, VecDeque<usize>)> {
    todo!()
}

pub fn render_solution(graph: CellGraph, stack: VecDeque<usize>) -> anyhow::Result<String> {
    todo!()
}

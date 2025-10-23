use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::anyhow;

use maze_robot::{Direction, Maze, Robot};

type CellGraph = HashMap<usize, [Option<usize>; 4]>;

const DIR_ARR: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

pub fn find_solution<M: Maze>(robot: Robot<M>) -> anyhow::Result<(CellGraph, VecDeque<usize>)> {
    let mut nxt_idx = 0;
    let mut location = nxt_idx;

    let mut stack = VecDeque::from([nxt_idx]);
    let mut graph: CellGraph = HashMap::new();
    let mut visited = HashSet::new();

    for

    Err(anyhow!("No solution found 😢"))
}

pub fn render_solution(graph: CellGraph, stack: VecDeque<usize>) -> anyhow::Result<String> {
    todo!()
}

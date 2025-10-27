use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
};

use anyhow::{Context, anyhow};

use maze_robot::{CARD_DIR_ARR, CardinalDirection, Cell, Direction, Maze, Robot};

type CellKey = usize;
type Parents = HashMap<CellKey, (CardinalDirection, CellKey)>;
type CellGraph = HashMap<CellKey, [Option<CellKey>; 4]>;

pub struct Solution {
    finish: CellKey,
    graph: CellGraph,
    parents: Parents,
}

pub fn render_solution(solution: Solution) -> String {
    todo!()
}

pub fn find_solution<M: Maze>(mut robot: Robot<M>) -> anyhow::Result<Solution> {
    // track parent direction & id by child
    // track visited nodes
    // track graph as node to adj list

    todo!()
}

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::anyhow;

use maze_robot::{Cell, Direction, Maze, Robot};

type CellGraph = HashMap<usize, Vec<(usize, Direction)>>;

const DIR_ARR: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

pub fn find_solution<M: Maze>(robot: Robot<M>) -> anyhow::Result<(CellGraph, VecDeque<usize>)> {
    let mut nxt_idx: usize = 0;
    let mut location = nxt_idx;

    let mut stack = VecDeque::from([(nxt_idx, None)]);
    let mut graph: CellGraph = HashMap::new();
    let mut visited = HashSet::new();

    while let Some(&(idx, dir)) = stack.front() {
        // mark visited
        visited.insert(idx);
        // pop neighbors?
        let parent = stack.get(1).map(|(parent_idx, parent_dir)| {
            (
                parent_idx,
                parent_dir
                    .expect("direction will exist if parent exists")
                    .reverse(),
            )
        });
        let neighbors = DIR_ARR.iter().fold(
            parent.map_or(HashMap::new(), |(p, d)| HashMap::from([(d, p)])),
            |acc, dir| {
                if let None = acc.get(dir) {
                    match robot.peek(dir) {
                        Cell::Finish | Cell::Open => {
                            let cell = nxt_idx;
                            nxt_idx += 1;
                            acc.insert(dir, cell)
                        }
                        Cell::Wall => acc,
                    }
                } else {
                    acc
                }
            },
        );
    }

    Err(anyhow!("No solution found 😢"))
}

pub fn render_solution(graph: CellGraph, stack: VecDeque<usize>) -> anyhow::Result<String> {
    todo!()
}

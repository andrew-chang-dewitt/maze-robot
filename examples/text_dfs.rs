use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::read_to_string,
};

use anyhow::anyhow;
use clap::Parser;

use maze_robot::{Cell, DIR_ARR, Direction, new_bot, text_maze::TextRobot, traits::Robot};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    maze_file: String,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let maze_text = read_to_string(app.maze_file)?;
    let robot: TextRobot = new_bot(maze_text)?;
    let solution = dfs_path(robot)?;

    println!("Solution:\n{}", render_solution(solution));

    Ok(())
}

fn render_solution(solution: Solution) -> String {
    solution.seen.to_string()
}

// TODO: should visual solution be a compile-time feature?
fn dfs_path(robot: TextRobot) -> anyhow::Result<Solution> {
    let mut visited = HashSet::new();
    let mut seen = RefCell::new(Seen::new());

    match dfs_helper(&robot, Node::default(), &mut visited, &mut seen) {
        Ok(()) => Err(anyhow!("No path to the finish was found!")),
        Err(MaybePath::Error(e)) => Err(e.context("Error encountered while searching for finish.")),
        Err(MaybePath::Done(path)) => Ok(Solution {
            winner: path.into_iter().rev().collect(),
            seen: seen.take(),
        }),
    }
}

#[derive(Eq, Debug, PartialEq)]
struct Solution {
    winner: Vec<Key>,
    seen: Seen,
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.seen.to_string())
    }
}

#[derive(Eq, Debug, Default, PartialEq)]
struct Seen {
    key_ord: Vec<Key>,
    key_map: HashMap<Key, Cell>,
    min_x: isize,
    max_x: isize,
    min_y: isize,
    max_y: isize,
}

impl Seen {
    fn new() -> Self {
        Seen {
            key_ord: Vec::new(),
            key_map: HashMap::new(),
            max_x: 0,
            min_x: 0,
            max_y: 0,
            min_y: 0,
        }
    }

    fn push(&mut self, key: Key, cell: Cell) {
        self.key_ord.push(key);
        self.key_map.insert(key, cell);
        self.max_x = max(self.max_x, key.0);
        self.min_x = min(self.min_x, key.0);
        self.max_y = max(self.max_y, key.1);
        self.min_y = min(self.min_y, key.1);

        // #[cfg(feature = "verbose")]
        // {
        //     eprintln!("[Seen::push] added {key}:{cell:?}");
        //     eprintln!("{self}")
        // }
    }

    fn get_by_coords(&self, x: isize, y: isize) -> Option<&Cell> {
        self.key_map.get(&Key(x, y))
    }

    fn _get_width(&self) -> isize {
        self.max_y - self.min_y
    }

    fn _get_height(&self) -> isize {
        self.max_x - self.min_x
    }
}

impl Display for Seen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut as_str = String::new();

        for y in (self.min_y..(self.max_y + 1)).rev() {
            for x in self.min_x..(self.max_x + 1) {
                match self.get_by_coords(x, y) {
                    // ▢◌○◍▦▧▩▨◈
                    Some(Cell::Open) => as_str.push_str("·"),
                    Some(Cell::Wall) => as_str.push_str("░"),
                    Some(Cell::Finish) => as_str.push_str("F"),
                    _ => as_str.push_str(" "),
                }
            }

            as_str.push_str("\n");
        }

        write!(f, "{as_str}")
    }
}

fn dfs_helper(
    robot: &TextRobot,
    node: Node,
    visited: &mut HashSet<Key>,
    seen: &RefCell<Seen>,
) -> Result<(), MaybePath> {
    // #[cfg(any(test, feature = "verbose"))]
    // {
    //     eprintln!("[dfs_helper] BEGIN w/\n{robot},\n{node:?}, & {visited:?}\n")
    // }
    let Node {
        key,
        cell,
        direction,
    } = node;
    // move robot if direction provided (otherwise at start)
    if let Some(dir) = direction {
        robot.go(dir).map_err(|e| MaybePath::Error(e.into()))?;
    }
    // handle FINISH case
    if let Cell::Finish = cell {
        // return early as error to signal done to try_fold
        return Err(MaybePath::Done(vec![key]));
    }

    // otherwise, continue
    // mark visited
    visited.insert(key);
    // for each neighbor
    DIR_ARR
        .iter()
        // peek in each direction
        .map(|&dir| {
            // #[cfg(feature = "verbose")]
            // {
            //     eprintln!("looking to the {dir}");
            // }
            (
                dir,
                robot.peek(dir).expect("this is infalliable w/ only 1 bot"),
            )
        })
        // track what we've seen
        .map(|(dir, cell)| {
            // calculate key
            let key = key.compute_in_dir(&dir);
            seen.borrow_mut().push(key, cell);
            // pass item down the iter chain
            return (dir, key, cell);
        })
        // filter out walls, preparing rest for recurring into
        .filter_map(|(dir, key, cell)| match cell {
            Cell::Wall => None,
            _ => Some(Node {
                key,
                cell,
                direction: Some(dir),
            }),
        })
        .try_fold((), |_, node| {
            let node_key = node.key;
            let node_direction = node.direction;
            // #[cfg(test)]
            // {
            //     eprintln!("[dfs_helper] handling neighbor {node:?}\n")
            // }
            // if in visited, skip node
            if visited.contains(&node_key) {
                // #[cfg(test)]
                // {
                //     eprintln!("[dfs_helper] skipping neighbor in visited")
                // }
                return Ok(());
            }
            // recurse into the neighboring node
            let recur_res = dfs_helper(robot, node, visited, seen);
            match recur_res {
                // handle done
                Err(MaybePath::Done(mut path)) => {
                    // push current position to path
                    path.push(key);
                    // #[cfg(test)]
                    // {
                    //     eprintln!("[dfs_helper] Finish found! building solution path: {path:?}")
                    // }
                    // end iteration early & propagate solution upward
                    // by returning solution as Err
                    Err(MaybePath::Done(path))
                }
                // if not done, move robot back to current cell
                // (reverse of direction used to enter the node)
                // then continue iteration/recursion
                Ok(()) => {
                    // #[cfg(test)]
                    // {
                    //     eprintln!("[dfs_helper] Solution not found through this node, moving back up one node.")
                    // }
                    if let Some(dir) = node_direction {
                        let new_dir = dir.reverse();
                        robot.go(new_dir).map_err(|e| MaybePath::Error(e.into()))
                    } else {
                        Ok(())
                    }
                }
                // propagate errors
                _ => {
                    #[cfg(test)]
                    {
                        eprintln!("[dfs_helper] Error encountered! propagating upward...")
                    }
                    recur_res
                }
            }
        })
}

enum MaybePath {
    Done(Vec<Key>),
    Error(anyhow::Error),
}

#[derive(Debug)]
struct Node {
    key: Key,
    cell: Cell,
    direction: Option<Direction>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            key: Key(0, 0),
            cell: Cell::Open,
            direction: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct Key(isize, isize);

impl Key {
    pub fn compute_in_dir(&self, direction: &Direction) -> Self {
        match direction {
            Direction::North => Self(self.0, self.1 + 1),
            Direction::South => Self(self.0, self.1 - 1),
            Direction::East => Self(self.0 + 1, self.1),
            Direction::West => Self(self.0 - 1, self.1),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    //  . 0 1 2
    //  0 S   F
    // -1 +   +
    // -2     +
    // -3 +   +
    // all: (0,0),(1,0),(1,-1),(1,-2),(0,-2),(1,-3),(2,0)
    // solution: (0,0),(1,0),(2,0)
    const MULTI_BRANCH_A: &str = "\
S F
+ +
  +
+ +";
    //  . 0 1 2 3 4 5 6
    //  2 + + + + + + +
    //  1 +       +   F
    //  0 S   + + +   +
    // -1 +           +
    // -2 + + + + + + +
    // solution:
    // (0,0),(1,0),(1,-1),(2,-1),(3,-1),(4,-1),(5,-1),(5,0),(5,1),(6,1)
    const MULTI_BRANCH_B: &str = "\
+++++++
+   + F
S +++ +
+     +
+++++++
";

    #[rstest]
    #[case("SF",vec![Key(0,0),Key(1,0)])]
    #[case("S +\n+ F",vec![Key(0,0),Key(1,0),Key(1,-1),Key(2,-1)])]
    fn can_solve_single_path_mazes(#[case] maze: &str, #[case] exp: Vec<Key>) {
        let robot: TextRobot = new_bot(maze).expect("robot to initialize");
        let act = dfs_path(robot).expect("solution to be found");

        assert_eq!(act.winner, exp)
    }

    #[rstest]
    #[case("S F\n+ +",vec![Key(0,0),Key(1,0),Key(2,0)])]
    #[case(MULTI_BRANCH_A,vec![Key(0,0),Key(1,0),Key(2,0)])]
    #[case(MULTI_BRANCH_B,vec![Key(0,0),Key(1,0),Key(1,-1),Key(2,-1),Key(3,-1),Key(4,-1),Key(5,-1),Key(5,0),Key(5,1),Key(6,1)])]
    fn can_solve_deadend_path_mazes(#[case] maze: &str, #[case] exp: Vec<Key>) {
        let robot: TextRobot = new_bot(maze).expect("robot to initialize");
        let act = dfs_path(robot).expect("solution to be found");

        assert_eq!(act.winner, exp)
    }
}

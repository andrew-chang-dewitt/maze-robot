use std::{collections::HashSet, fmt::Display};

use anyhow::anyhow;

use crate::{Cell, DIR_ARR, Direction, MazeError, Robot};

pub fn solve<M: TryInto<Robot, Error = MazeError>>(maze: M) -> anyhow::Result<Vec<Key>> {
    // set up robot w/ given maze
    let robot = maze.try_into()?;

    // find solution w/ dfs
    dfs_path(robot)
}

fn dfs_path(robot: Robot) -> anyhow::Result<Vec<Key>> {
    let mut visited = HashSet::new();

    match dfs_helper(&robot, Node::default(), &mut visited) {
        Ok(()) => Err(anyhow!("No path to the finish was found!")),
        Err(Solution::Error(e)) => Err(e.context("Error encountered while searching for finish.")),
        Err(Solution::Done(path)) => Ok(path.into_iter().rev().collect()),
    }
}

fn dfs_helper(robot: &Robot, node: Node, visited: &mut HashSet<Key>) -> Result<(), Solution> {
    #[cfg(test)]
    {
        println!("[dfs_helper] BEGIN w/\n{robot},\n{node:?},\n& {visited:?}\n")
    }
    let Node {
        key,
        cell,
        direction,
    } = node;
    // move robot if direction provided (otherwise at start)
    if let Some(dir) = direction {
        robot.go(dir).map_err(|e| Solution::Error(e.into()))?;
    }
    // handle FINISH case
    if let Cell::Finish = cell {
        // return early as error to signal done to try_fold
        return Err(Solution::Done(vec![key]));
    }

    // otherwise, continue
    // mark visited
    visited.insert(key);
    // for each neighbor
    DIR_ARR
        .iter()
        // peek in each direction
        .map(|&dir| (dir, robot.peek(dir)))
        // filter out walls, preparing rest for recurring into
        .filter_map(|(dir, cell)| match cell {
            Cell::Wall => None,
            _ => {
                let next = key.compute_in_dir(&dir);
                Some(Node {
                    key: next,
                    cell,
                    direction: Some(dir),
                })
            }
        })
        .try_fold((), |_, node| {
            let node_key = node.key;
            let node_direction = node.direction;
            #[cfg(test)]
            {
                println!("[dfs_helper] handling neighbor {node:?}\n")
            }
            // if in visited, skip node
            if visited.contains(&node_key) {
                #[cfg(test)]
                {
                    println!("[dfs_helper] skipping neighbor in visited")
                }
                return Ok(());
            }
            // recurse into the neighboring node
            let recur_res = dfs_helper(robot, node, visited);
            match recur_res {
                // handle done
                Err(Solution::Done(mut path)) => {
                    // push current position to path
                    path.push(key);
                    #[cfg(test)]
                    {
                        println!("[dfs_helper] Finish found! building solution path: {path:?}")
                    }
                    // end iteration early & propagate solution upward
                    // by returning solution as Err
                    Err(Solution::Done(path))
                }
                // if not done, move robot back to current cell
                // (reverse of direction used to enter the node)
                // then continue iteration/recursion
                Ok(()) => {
                    #[cfg(test)]
                    {
                        println!("[dfs_helper] Solution not found through this node, moving back up one node.")
                    }
                    if let Some(dir) = node_direction {
                        let new_dir = dir.reverse();
                        robot.go(new_dir).map_err(|e| Solution::Error(e.into()))
                    } else {
                        Ok(())
                    }
                }
                // propagate errors
                _ => { 
                    #[cfg(test)]
                    {
                        println!("[dfs_helper] Error encountered! propagating upward...")
                    }
                    recur_res
                },
            }
        })
}

enum Solution {
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
pub struct Key(isize, isize);

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
        let act = solve(maze).expect("solution to be found");

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case("S F\n+ +",vec![Key(0,0),Key(1,0),Key(2,0)])]
    #[case(MULTI_BRANCH_A,vec![Key(0,0),Key(1,0),Key(2,0)])]
    #[case(MULTI_BRANCH_B,vec![Key(0,0),Key(1,0),Key(1,-1),Key(2,-1),Key(3,-1),Key(4,-1),Key(5,-1),Key(5,0),Key(5,1),Key(6,1)])]
    fn can_solve_deadend_path_mazes(#[case] maze: &str, #[case] exp: Vec<Key>) {
        let act = solve(maze).expect("solution to be found");

        assert_eq!(act, exp)
    }
}

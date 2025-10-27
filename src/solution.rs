use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use anyhow::anyhow;
use maze_robot::{CARD_DIR_ARR, CardinalDirection, Cell, Maze, Robot, RobotError};

type CellKey = (isize, isize);
type Parents = HashMap<CellKey, (CardinalDirection, CellKey)>;
type CellGraph = HashMap<CellKey, [Option<CellKey>; 4]>;

pub struct Solution {
    _finish: CellKey,
    _graph: CellGraph,
    _parents: Parents,
}

pub fn render_solution(_solution: Solution) -> String {
    todo!()
}

const _START: (isize, isize) = (0, 0);

pub fn find_solution<M: Maze>(_robot: Robot<M>) -> anyhow::Result<Solution> {
    // track parent direction & id by child
    // track visited nodes
    // track graph as node to adj list

    todo!()
}

/// graph of K
trait GraphN<K>
where
    K: Eq + Copy + Debug + Hash + Sized,
{
    fn dfs(
        &self,
        root: &K,
        on_visit: &mut impl FnMut(K) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let mut visited = HashSet::new();

        self.dfs_helper(root, on_visit, &mut visited)
    }

    fn dfs_helper(
        &self,
        key: &K,
        on_visit: &mut impl FnMut(K) -> anyhow::Result<()>,
        visited: &mut HashSet<K>,
    ) -> anyhow::Result<()> {
        println!("[dfs_helper] key: {key:?}, visited: {visited:?}");
        on_visit(*key)?;
        visited.insert(*key);

        // call get_neighbors here to get list
        let neighbors = self.get_neighbors(key);
        for neighbor in neighbors {
            if !visited.contains(&neighbor) {
                self.dfs_helper(&neighbor, on_visit, visited)?;
            }
        }

        Ok(())
    }

    fn get_neighbors(&self, key: &K) -> impl Iterator<Item = K>;
}

struct MazeSolver<F, G>
where
    F: Fn(CardinalDirection) -> Cell,
    G: FnMut(CardinalDirection) -> Result<(), RobotError>,
{
    peek_fn: F,
    move_fn: G,
}

impl<F, G> GraphN<CellKey> for MazeSolver<F, G>
where
    F: Fn(CardinalDirection) -> Cell,
    G: FnMut(CardinalDirection) -> Result<(), RobotError>,
{
    fn get_neighbors(&self, key: &CellKey) -> impl Iterator<Item = CellKey> {
        CARD_DIR_ARR
            .iter()
            .filter_map(|&dir| match (self.peek_fn)(dir) {
                Cell::Finish | Cell::Open => Some(todo!(
                    " need to calculate new cell key from direction and current key"
                )),
                Cell::Wall => None,
            })
    }
}

struct GraphDeg4<K>
where
    K: Eq + Debug + Hash + Sized,
{
    adj_list: HashMap<K, [Option<K>; 4]>,
}

impl<K> GraphDeg4<K> where K: Eq + Copy + Debug + Hash + Sized {}

impl<K> GraphN<K> for GraphDeg4<K>
where
    K: Eq + Copy + Debug + Hash + Sized,
{
    // TODO: use this to peek at TextMaze & get neighbors from result!
    fn get_neighbors(&self, key: &K) -> impl Iterator<Item = K> {
        let neighbors = self
            .adj_list
            .get(key)
            .ok_or(anyhow!("Key {key:?} must exist"))?;

        let res = neighbors
            .iter()
            .filter_map(|maybe_neighbor| match maybe_neighbor {
                &Some(n) => Some(n),
                None => None,
            });

        Ok(res)
    }
}

impl<T, K> From<T> for GraphDeg4<K>
where
    K: Eq + Copy + Debug + Hash + Sized,
    T: Into<HashMap<K, [Option<K>; 4]>>,
{
    fn from(value: T) -> Self {
        Self {
            adj_list: value.into(),
        }
    }
}

#[cfg(test)]
mod graph_tests {
    use super::*;

    #[test]
    fn dfs_can_visit_all_nodes() {
        // test graph of degree=4
        // 0 --- 3
        //       |
        //       2 --- 1
        let graph = GraphDeg4::from([
            (0, [None, Some(3), None, None]),
            (1, [None, None, None, Some(2)]),
            (2, [Some(3), Some(1), None, None]),
            (3, [None, None, Some(2), Some(0)]),
        ]);

        // while traversing graph, push each node key to vec so it records order of traversal from
        // left to right
        let mut act = vec![];
        let mut visitor = |val| Ok(act.push(val));
        // should end up w/ 0 -> 3 -> 2 -> 1
        let exp = vec![0, 3, 2, 1];

        let res = graph.dfs(&0, &mut visitor);
        res.expect("dfs should succeed");

        assert_eq!(exp, act)
    }

    #[test]
    fn dfs_can_avoid_cycles() {
        // test graph of degree=4
        // 0 --- 3
        // |     |
        // 4 --- 2 --- 1
        let graph = GraphDeg4::from([
            (0, [None, Some(3), None, Some(4)]),
            (1, [None, None, None, Some(2)]),
            (2, [Some(3), Some(1), None, Some(4)]),
            (3, [None, None, Some(2), Some(0)]),
            (4, [Some(0), Some(2), None, None]),
        ]);

        // while traversing graph, push each node key to vec so it records order of traversal from
        // left to right
        let mut act = vec![];
        let mut visitor = |val| Ok(act.push(val));
        // should end up w/ 0 -> 3 -> 2 -> 1
        let exp = vec![0, 3, 2, 1, 4];

        let res = graph.dfs(&0, &mut visitor);
        res.expect("dfs should succeed");

        assert_eq!(exp, act)
    }
}

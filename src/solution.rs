use std::{collections::HashSet, error::Error, fmt::Debug, hash::Hash};

use maze_robot::{CARD_DIR_ARR, CardinalDirection, Cell, Maze, Robot, RobotError};

const START: CellKey = CellKey(0, 0);

pub fn render_solution(_solution: Vec<CellKey>) -> String {
    todo!()
}

pub fn find_solution<M: Maze>(robot: Robot<M>) -> anyhow::Result<Vec<CellKey>> {
    let peek_fn = |_: &CellKey, direction: &CardinalDirection| robot.peek(*direction);

    let solution = Solution { peek_fn: &peek_fn };

    // TODO: pick up here--this overflows because the robot never moves
    let mut visitor = |mut acc: Vec<CellKey>, loc: &CellKey| {
        acc.push(*loc);

        robot.go(direction).map(|_| acc).map_err(|e| e.into())
    };

    solution.dfs(&START, vec![START], &mut visitor)
}

// struct Solution<Peek,Move>
struct Solution<'a, Peek>
where
    Peek: Fn(&CellKey, &CardinalDirection) -> Cell,
{
    peek_fn: &'a Peek,
    // move_fn: &'a Move,
}

// trait Discoverable<Instr, Loc>
impl<'a, Peek> Discoverable<CardinalDirection, CellKey> for Solution<'a, Peek>
where
    Peek: Fn(&CellKey, &CardinalDirection) -> Cell,
{
    const N: usize = 4;
    const PEEK_INSTR_ARR: &'static [CardinalDirection] = &CARD_DIR_ARR;

    fn peek_one(&self, from: &CellKey, instr: &CardinalDirection) -> Option<CellKey> {
        let cell = (self.peek_fn)(from, instr);

        match cell {
            Cell::Open | Cell::Finish => Some(from.compute(*instr)),
            Cell::Wall => None,
        }
    }
}

impl<'a, Peek> DFSTraversable<CellKey> for Solution<'a, Peek>
where
    Peek: Fn(&CellKey, &CardinalDirection) -> Cell,
{
    type Error = anyhow::Error;

    fn get_neighbors(&self, key: &CellKey) -> impl Iterator<Item = CellKey> {
        self.discover_valid(key)
    }
}

#[cfg(test)]
mod solution_tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::trivial_maze("+++\nS F\n+++", CardinalDirection::East, vec![CellKey(0,0), CellKey(1,0), CellKey(2,0)])]
    fn can_solve_mazes(
        #[case] maze: &str,
        #[case] dir: CardinalDirection,
        #[case] exp: Vec<CellKey>,
    ) {
        let robot = Robot::try_from((maze, dir)).expect("robot to be created");

        let act = find_solution(robot).expect("solution to be found");

        assert_eq!(exp, act);
    }
}

/// unique identifiers for maze cells (x,y) coords w/ start @ (0,0)
#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct CellKey(isize, isize);

impl CellKey {
    fn compute(&self, direction: CardinalDirection) -> Self {
        let &CellKey(x, y) = self;

        match direction {
            CardinalDirection::North => CellKey(x + 1, y),
            CardinalDirection::East => CellKey(x, y + 1),
            CardinalDirection::South => CellKey(x - 1, y),
            CardinalDirection::West => CellKey(x, y - 1),
        }
    }
}

/// graph of K
trait DFSTraversable<K>
where
    K: Eq + Copy + Debug + Hash + Sized,
{
    type Error;

    fn dfs<T>(
        &self,
        root: &K,
        initial_val: T,
        accumulator_fn: &mut impl FnMut(T, &K) -> Result<T, Self::Error>,
    ) -> Result<T, Self::Error> {
        let mut visited = HashSet::from([*root]);

        self.dfs_helper(root, initial_val, accumulator_fn, &mut visited)
    }

    fn dfs_helper<T>(
        &self,
        key: &K,
        accumulator_val: T,
        accumulator_fn: &mut impl FnMut(T, &K) -> Result<T, Self::Error>,
        visited: &mut HashSet<K>,
    ) -> Result<T, Self::Error> {
        println!("[dfs_helper] key: {key:?}, visited: {visited:?}");
        self.get_neighbors(key)
            .try_fold(accumulator_val, |acc, neighbor| {
                if visited.contains(&neighbor) {
                    Ok(acc)
                } else {
                    visited.insert(neighbor);

                    accumulator_fn(acc, &neighbor).and_then(|new_acc| {
                        self.dfs_helper(&neighbor, new_acc, accumulator_fn, visited)
                    })
                }
            })
    }

    fn get_neighbors(&self, key: &K) -> impl Iterator<Item = K>;
}

#[cfg(test)]
mod dfs_tests {
    use std::collections::HashMap;

    use super::*;

    struct GraphDeg4<K>
    where
        K: Eq + Debug + Hash + Sized,
    {
        adj_list: HashMap<K, [Option<K>; 4]>,
    }

    impl<K> DFSTraversable<K> for GraphDeg4<K>
    where
        K: Eq + Copy + Debug + Hash + Sized,
    {
        type Error = ();

        fn get_neighbors(&self, key: &K) -> impl Iterator<Item = K> {
            self.adj_list
                .get(key)
                .expect("Key {key:?} must exist")
                .iter()
                .filter_map(|maybe_neighbor| maybe_neighbor.to_owned())
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
        let mut visitor = |mut acc: Vec<usize>, val: &usize| {
            acc.push(*val);
            Ok(acc)
        };
        // should end up w/ 0 -> 3 -> 2 -> 1
        let exp = vec![0, 3, 2, 1];

        // call dfs traversal w/ visitor & compare result
        let act: Vec<usize> = graph
            .dfs(&0, vec![0], &mut visitor)
            .expect("dfs should succeed");
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
        let mut visitor = |mut acc: Vec<usize>, val: &usize| {
            acc.push(*val);
            Ok(acc)
        };
        // should end up w/ 0 -> 3 -> 2 -> 1
        let exp = vec![0, 3, 2, 1, 4];

        // call dfs traversal w/ visitor & compare result
        let act: Vec<usize> = graph
            .dfs(&0, vec![0], &mut visitor)
            .expect("dfs should succeed");
        assert_eq!(exp, act)
    }
}

trait Discoverable<Instr, Loc>
where
    Instr: 'static + Sized,
{
    const N: usize;
    const PEEK_INSTR_ARR: &'static [Instr];

    fn peek_one(&self, from: &Loc, instr: &Instr) -> Option<Loc>;

    fn discover_valid(&self, from: &Loc) -> impl Iterator<Item = Loc> {
        Self::PEEK_INSTR_ARR
            .iter()
            .filter_map(|instr| self.peek_one(from, instr))
    }
}

#[cfg(test)]
mod discoverable_tests {
    use std::collections::HashMap;

    use super::*;

    struct Explorer<'a, Peek>
    where
        Peek: Fn(&usize, &usize) -> isize,
    {
        peek_fn: &'a Peek,
    }

    impl<'a, Peek> Discoverable<usize, usize> for Explorer<'a, Peek>
    where
        Peek: Fn(&usize, &usize) -> isize,
    {
        const N: usize = 4;
        const PEEK_INSTR_ARR: &'static [usize] = &[0, 1, 2, 3];

        fn peek_one(&self, from: &usize, instr: &usize) -> Option<usize> {
            (self.peek_fn)(from, instr).try_into().ok()
        }
    }

    impl<'a, Peek> DFSTraversable<usize> for Explorer<'a, Peek>
    where
        Peek: Fn(&usize, &usize) -> isize,
    {
        type Error = ();

        fn get_neighbors(&self, key: &usize) -> impl Iterator<Item = usize> {
            self.discover_valid(key)
        }
    }

    #[test]
    fn discoverable_can_direct_how_to_explore_unknown_graph() {
        // test graph of degree=4
        // 0 --- 3
        // |     |
        // 4 --- 2 --- 1
        let maze = HashMap::from([
            (0, [-1, 3, -1, 4]),
            (1, [-1, -1, -1, 2]),
            (2, [3, 1, -1, 4]),
            (3, [-1, -1, 2, 0]),
            (4, [0, 2, -1, -1]),
        ]);
        let mut loc: usize = 0;
        let peek_fn = |from: &usize, dir: &usize| {
            maze.get(from)
                .expect(format!("key {} must exist!", from).as_str())[*dir]
        };
        let explorer = Explorer { peek_fn: &peek_fn };

        // while traversing graph, push each node key to vec so it records order of traversal from
        // left to right
        let mut visitor = |mut acc: Vec<usize>, val: &usize| {
            loc = *val;
            acc.push(*val);

            Ok(acc)
        };
        // should end up w/ 0 -> 3 -> 2 -> 1
        let exp = vec![0, 3, 2, 1, 4];

        let act: Vec<usize> = explorer
            .dfs(&0, vec![0], &mut visitor)
            .expect("dfs should succeed");

        assert_eq!(exp, act)
    }
}

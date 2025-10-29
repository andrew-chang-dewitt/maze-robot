use std::{collections::HashSet, fmt::Debug, hash::Hash};

use maze_robot::{CARD_DIR_ARR, CardinalDirection, Cell, Maze, Robot};

const START: CellKey = CellKey(0, 0);

pub fn render_solution(_solution: Vec<CellKey>) -> String {
    todo!()
}

pub fn find_solution<Loc: 'static + Eq + Copy + Debug + Hash + PartialEq, M: Maze<Loc>>(
    robot: Robot<M, Loc>,
    start: Loc,
) -> anyhow::Result<Vec<CellKey>> {
    let mut state = start;

    // TODO: pick up here--this overflows because the robot never moves
    let mut visitor = |mut acc: Vec<CellKey>, value: &IterState<CellKey, Loc>| {
        let IterState(key, _, direction) = value;
        acc.push(*key);

        if let Some(dir) = direction {
            robot
                .go(&state, *dir)
                .map(|new_state| {
                    state = new_state;

                    acc
                })
                .map_err(|e| e.into())
        } else {
            Ok(acc)
        }
    };

    robot.dfs(KeyState(START, start), Vec::<CellKey>::new(), &mut visitor)
}

impl<M, S> Discoverable for Robot<M, S>
where
    S: 'static + Eq + Copy + Debug + Hash,
    M: Maze<S>,
{
    const N: usize = 4;
    const PEEK_INSTR_ARR: &'static [CardinalDirection] = &CARD_DIR_ARR;

    type Key = KeyState<CellKey, S>;
    type Instr = CardinalDirection;
    type Item = IterState<CellKey, S>;

    fn peek_one(
        &self,
        KeyState(from_key, from_state): KeyState<CellKey, S>,
        instr: &CardinalDirection,
    ) -> Option<Self::Item> {
        let peeked = self.peek(&from_state, *instr);

        match peeked {
            (Cell::Open, Some(peek_state)) | (Cell::Finish, Some(peek_state)) => Some(IterState(
                from_key.compute(*instr),
                peek_state,
                Some(*instr),
            )),
            (Cell::Wall, _) | (_, None) => None,
        }
    }
}

impl<T> DFSTraverser for T
where
    T: Discoverable,
{
    type Key = <T as Discoverable>::Key;
    type Item = <T as Discoverable>::Item;

    fn get_neighbors(&self, from: Self::Key) -> impl Iterator<Item = Self::Item> {
        self.discover_valid(from)
    }
}

/// unique identifiers for maze cells (x,y) coords w/ start @ (0,0)
#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct CellKey(isize, isize);

impl CellKey {
    fn compute(&self, direction: CardinalDirection) -> Self {
        let &CellKey(x, y) = self;

        match direction {
            CardinalDirection::North => CellKey(x, y + 1),
            CardinalDirection::East => CellKey(x + 1, y),
            CardinalDirection::South => CellKey(x, y - 1),
            CardinalDirection::West => CellKey(x - 1, y),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct KeyState<T: Copy + Debug, U: Copy + Debug>(T, U);
#[derive(Copy, Clone, Debug)]
struct IterState<T: Copy + Debug, U: Copy + Debug>(T, U, Option<CardinalDirection>);

impl<T: Copy + Debug, U: Copy + Debug> From<KeyState<T, U>> for IterState<T, U> {
    fn from(value: KeyState<T, U>) -> Self {
        IterState(value.0, value.1, None)
    }
}

impl<T: Copy + Debug, U: Copy + Debug> From<IterState<T, U>> for KeyState<T, U> {
    fn from(value: IterState<T, U>) -> Self {
        KeyState(value.0, value.1)
    }
}

#[cfg(test)]
mod solution_tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::trivial_maze("+++\nS F\n+++", vec![CellKey(0,0), CellKey(1,0), CellKey(2,0)])]
    #[case::dead_end_paths("+++\nS F\n+ +\n+++", vec![CellKey(0,0), CellKey(1,0), CellKey(2,0)])]
    fn can_solve_mazes(#[case] maze: &str, #[case] exp: Vec<CellKey>) {
        let (robot, start) = Robot::try_new(maze).expect("robot to be created");

        let act = find_solution(robot, start).expect("solution to be found");

        assert_eq!(exp, act);
    }
}

struct Graph<Key, Data> {
    node: Key,
    data: Data,
    parent: &Option<GraphIter<Key>>,
}

impl Iterator for Iter<Key> {
    type Item = Key;
}

/// A container that can be traversed using depth-first-search, as though it were a graph.
///
/// As it is traversed, the given `accumulator_fn` (taking the accumulated value so far, & the next
/// value yielded during iteration) is called for each node. This accumulator can
/// short-circuit the iteration by returning an `Err` value; otherwise, it returns an `Ok`
/// containing the accumulated value.
trait DFSTraverser {
    type Key: Eq + Copy + Debug + Hash + Sized + From<Self::Item>;
    type Item: From<Self::Key> + Copy;

    // TODO: let's try making this a type of iterator instead!
    // then we can use std iterator adapter methods
    // needs to somehow construct a struct, maybe GraphIter for tracking state & fetching next...
    fn dfs_iter(self) -> impl Iterator<Item = Self::Item> {
        todo!()
    }

    fn dfs<T, E>(
        &self,
        root: Self::Key,
        initial_val: T,
        accumulator_fn: &mut impl FnMut(T, &Self::Item) -> Result<T, E>,
    ) -> Result<T, E> {
        let accumulated = accumulator_fn(initial_val, &root.into())?;
        let mut visited = HashSet::from([root]);

        self.dfs_helper(root, accumulated, accumulator_fn, &mut visited)
    }

    fn dfs_helper<T, E>(
        &self,
        key: Self::Key,
        accumulator_val: T,
        accumulator_fn: &mut impl FnMut(T, &Self::Item) -> Result<T, E>,
        visited: &mut HashSet<Self::Key>,
    ) -> Result<T, E> {
        println!("[dfs_helper] key: {:?}, visited: {visited:?}", key);
        self.get_neighbors(key)
            .try_fold(accumulator_val, |acc, neighbor| {
                if visited.contains(&neighbor.into()) {
                    Ok(acc)
                } else {
                    visited.insert(neighbor.into());

                    accumulator_fn(acc, &neighbor).and_then(|new_acc| {
                        self.dfs_helper(neighbor.into(), new_acc, accumulator_fn, visited)
                    })
                }
            })
    }

    fn get_neighbors(&self, from: Self::Key) -> impl Iterator<Item = Self::Item>;
}

#[cfg(test)]
mod dfs_tests {
    use std::collections::HashMap;

    use super::*;

    struct GraphDeg4<Key>
    where
        Key: Eq + Debug + Hash + Sized,
    {
        adj_list: HashMap<Key, [Option<Key>; 4]>,
    }

    impl DFSTraverser for GraphDeg4<usize> {
        type Key = usize;
        type Item = Self::Key;

        fn get_neighbors(&self, key: Self::Key) -> impl Iterator<Item = Self::Item> {
            self.adj_list
                .get(&key)
                .expect("Key {key:?} must exist")
                .iter()
                .filter_map(|maybe_neighbor| maybe_neighbor.to_owned())
        }
    }

    impl<T, Key> From<T> for GraphDeg4<Key>
    where
        Key: Eq + Copy + Debug + Hash + Sized,
        T: Into<HashMap<Key, [Option<Key>; 4]>>,
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
            Ok::<Vec<usize>, ()>(acc)
        };
        // should end up w/ 0 -> 3 -> 2 -> 1
        let exp = vec![0, 3, 2, 1];

        // call dfs traversal w/ visitor & compare result
        let act: Vec<usize> = graph
            .dfs(0, vec![], &mut visitor)
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
        let act = graph
            .dfs::<Vec<usize>, ()>(0, vec![], &mut visitor)
            .expect("dfs should succeed");
        assert_eq!(exp, act)
    }
}

trait Discoverable {
    type Key: 'static + Eq + Copy + Debug + Hash + Sized + From<Self::Item>;
    type Instr: 'static;
    type Item: From<Self::Key> + Copy;

    const N: usize;
    const PEEK_INSTR_ARR: &'static [Self::Instr];

    fn peek_one(&self, from: Self::Key, instr: &Self::Instr) -> Option<Self::Item>;

    fn discover_valid(&self, from: Self::Key) -> impl Iterator<Item = Self::Item> {
        Self::PEEK_INSTR_ARR
            .iter()
            .filter_map(move |instr| self.peek_one(from, instr))
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

    impl<'a, Peek> Discoverable for Explorer<'a, Peek>
    where
        Peek: Fn(&usize, &usize) -> isize,
    {
        const N: usize = 4;
        const PEEK_INSTR_ARR: &'static [usize] = &[0, 1, 2, 3];

        type Key = usize;
        type Instr = usize;
        type Item = usize;

        fn peek_one(&self, from: usize, instr: &usize) -> Option<Self::Item> {
            (self.peek_fn)(&from, instr).try_into().ok()
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

            Ok::<Vec<usize>, ()>(acc)
        };
        // should end up w/ 0 -> 3 -> 2 -> 1
        let exp = vec![0, 3, 2, 1, 4];

        let act: Vec<usize> = explorer
            .dfs(0, vec![], &mut visitor)
            .expect("dfs should succeed");

        assert_eq!(exp, act)
    }
}

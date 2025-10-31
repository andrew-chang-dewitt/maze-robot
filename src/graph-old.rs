use std::{fmt::Debug, marker::PhantomData};

/// A structural graph implementation.

/// A graph consists of a root Node.
struct Graph<G: GraphNode> {
    root: G,
}

/// The composition of a graph Node, or vertex.
///
/// Consists of a unique identifier (`Key`), any associated `Data`, & a reference to a parent Node.
#[derive(Debug)]
struct Node<Key: Eq + PartialEq, Data> {
    key: Key,
    data: Data,
}

/// Node equality is decided soley on key equality w/ no regard for any associated node data.
impl<Key: Eq, Data> PartialEq for Node<Key, Data> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

/// The behavioural description of a structurally recursive Graph.
///
/// Describes how a graph Node knows what its neighbors are, as well as how to get back to its
/// parent Node, if it has one.
///
/// Additionally describes how to get graph iterators using different traversal orders from the
/// node the method is called upon.
///
/// > TODO
/// >
/// > - [ ] implement bfs traversal order iterator
/// > - [ ] dijkstra's? as an iterator w/ some given end condition
trait GraphNode: Eq {
    type Item;

    fn get_neighbors(&self) -> impl Iterator<Item = Self::Item>;
}

/// Stores state for iterating over a graph.
struct GraphIter<'a, T, G, Order>
where
    G: GraphNode,
    Order: GraphTraversalOrder,
{
    _next: T,
    _lifetime: PhantomData<&'a G>,
    _order: PhantomData<Order>,
}

impl<'a, T, G, Order> GraphIter<'a, T, G, Order>
where
    G: GraphNode,
    Order: GraphTraversalOrder,
{
    pub fn new(root: &'a G) -> Self {
        Self {
            _next: root,
            _lifetime: PhantomData,
            _order: PhantomData,
        }
    }
}

/// A marker trait simply to indicate that an implementing type is to be treated as an
/// implementable graph traversal order
trait GraphTraversalOrder {}

/// Zero-Sized Type to indicate a Depth-First-Search traversal order when iterating over a graph.
struct DFS;
impl GraphTraversalOrder for DFS {}

/// Zero-Sized Type to indicate a Breadth-First-Search traversal order when iterating over a graph.
struct BFS;
impl GraphTraversalOrder for BFS {}

impl<'a, G: Graph> Iterator for GraphIter<'a, G, DFS> {
    type Item = G;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::{fixture, rstest};

    use super::*;

    // An example graph of max degree of 4, modeled as an adjacency list w/ unused edges as -1 &
    // used edges as a positive node id
    //
    // 0 --- 1 --- 5 ---
    // | \_  |     | \_|
    // |   \ |    /
    // 2 --- 3   /
    // | \_  |  /
    // |   \ | /
    // 4     6
    //
    #[fixture]
    fn adj_list() -> HashMap<isize, [isize; 4]> {
        HashMap::from([
            (0, [1, 3, 2, -1]),
            (1, [5, 3, 0, -1]),
            (2, [0, 3, 6, 4]),
            (3, [1, 6, 2, 0]),
            (4, [2, -1, -1, -1]),
            (5, [5, 6, 1, -1]),
            (6, [3, 5, 2, -1]),
        ])
    }

    #[derive(Eq, Debug)]
    struct AdjNode<'a>(isize, &'a HashMap<isize, [isize; 4]>);

    impl<'a> PartialEq for AdjNode<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl<'a> GraphNode for AdjNode<'a> {
        type Item = isize;

        fn get_neighbors(&self) -> impl Iterator<Item = Self> {
            self.1
                .get(&self.0)
                .expect("node must exist!")
                .iter()
                .filter_map(|&i| {
                    if i >= 0 {
                        Some(AdjNode(i, self.1))
                    } else {
                        None
                    }
                })
        }
    }

    #[rstest]
    #[case(0,vec![1,3,2])]
    #[case(2,vec![0,3,6,4])]
    #[case(4,vec![2])]
    #[case(5,vec![5,6,1])]
    fn can_get_neighbors_using_adj_list(
        #[case] key: isize,
        #[case] exp: Vec<isize>,
        adj_list: HashMap<isize, [isize; 4]>,
    ) {
        let node = AdjNode(key, &adj_list);
        let act: Vec<isize> = node.get_neighbors().map(|adj| adj.0).collect();

        assert_eq!(act, exp);
    }

    // 0 --- 1 --- 5 ---   (0, [1, 3, 2, -1]),
    // | \_  |     | \_|   (1, [5, 3, 0, -1]),
    // |   \ |    /        (2, [0, 3, 6, 4]),
    // 2 --- 3   /         (3, [1, 6, 2, 0]),
    // | \_  |  /          (4, [2, -1, -1, -1]),
    // |   \ | /           (5, [5, 6, 1, -1]),
    // 4     6             (6, [3, 5, 2, -1]),
    #[rstest]
    #[case(0,vec![0,1,5,6,3,2,4])]
    #[case(1,vec![1,5,6,3,2,0,4])]
    #[case(2,vec![2,0,1,5,6,3,4])]
    #[case(3,vec![3,1,5,6,3,2,4])]
    #[case(4,vec![4,2,0,1,5,6,3])]
    #[case(5,vec![5,6,3,1,0,2,4])]
    #[case(6,vec![6,3,1,5,0,2,4])]
    fn can_iterate_using_dfs_order(
        #[case] key: isize,
        #[case] exp: Vec<isize>,
        adj_list: HashMap<isize, [isize; 4]>,
    ) {
        let node = AdjNode(key, &adj_list);
        let act: Vec<isize> = node.iter_dfs().map(|adj| adj.0).collect();

        assert_eq!(exp, act);
    }
}

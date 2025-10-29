use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// A structural graph implementation.

/// The composition of a graph Node, or vertex.
///
/// Consists of a unique identifier (`Key`), any associated `Data`, & a reference to a parent Node.
#[derive(Debug)]
struct Node<Key: Eq + PartialEq + Debug, Data> {
    key: Key,
    data: Data,
}

/// Node equality is decided soley on key equality w/ no regard for any associated node data.
impl<Key: Eq + Debug, Data> PartialEq for Node<Key, Data> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

/// The behavioural description of a structurally recursive Graph.
///
/// Describes how a graph Node knows what its neighbors are, as well as how to get back to its
/// parent Node, if it has one.
trait Graph {
    /// The node type for this graph. Must also be an implementer of Graph.
    type Item: Graph + Eq;

    fn get_neighbors(&self) -> impl Iterator<Item = Self::Item>;
    fn get_parent(&self) -> Option<Self::Item>;
}

/// Stores state for iterating over a graph.
struct GraphIter<'a, G: Graph, Order> {
    _next: G,
    _lifetime: PhantomData<&'a G::Item>,
    _order: PhantomData<Order>,
}

/// Zero-Sized Type to indicate a Depth-First-Search traversal order when iterating over a graph.
struct DFS;

/// Zero-Sized Type to indicate a Breadth-First-Search traversal order when iterating over a graph.
struct BFS;

impl<'a, G: Graph> Iterator for GraphIter<'a, G, DFS> {
    type Item = G::Item;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::fixture;

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

    #[derive(Eq)]
    struct AdjNode(isize, Option<isize>);

    impl PartialEq for AdjNode {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Graph for AdjNode {
        type Item = Self;

        fn get_parent(&self) -> Option<Self::Item> {
            todo!()
        }

        fn get_neighbors(&self) -> impl Iterator<Item = Self::Item> {
            todo!()
        }
    }

    #[rstest]
    fn can_get_neighbors_using_adj_list(graph: HashMap<isize, [isize; 4]>) {
        todo!()
    }
}

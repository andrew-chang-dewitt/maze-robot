//! Models a graph as a structurally recursive sum of a Node in context* & the Empty node.
//!
//! A Node is assumed to know only itself (a unique key & possibly any other data to store with
//! it)&mdash;meaning a Node has no concept of edges, neighbor, or other ideas of its place within
//! a Graph. Instead, a Node's context is held outside of it (in an object implementing behaviour
//! appropriately named `ContextType`) & together, they can be used to locate neighbors & traverse
//! the graph.

use std::marker::PhantomData;

/// The top-level sum-type container, indicating if a graph has members, or if it is empty. Most
/// graph behaviours are implemented here, including methods for getting an iterator over the
/// graph's nodes.
enum FunGraph<'a, Ctx, Key>
where
    Key: Eq,
    Ctx: ContextType<'a, Key>,
{
    Graph(Ctx, &'a Self, PhantomData<Key>),
    Empty,
}

impl<'a, Ctx, Key> FunGraph<'a, Ctx, Key>
where
    Key: Eq,
    Ctx: ContextType<'a, Key>,
{
    pub fn new() -> Self {
        Self::Empty
    }
}

trait GraphType<'a, Ctx, Key>
where
    Key: Eq,
    Ctx: ContextType<'a, Key>,
{
    // type Error;

    fn empty() -> Self;
    // fn add_node_with_ctx(&mut self, ctx: Ctx) -> &Self {}
    // fn add_edge(&mut self, from: Key, to: Key) -> Result<&Self, Self::Error>;

    // fn get(&self, key: Key) -> Option<&<Ctx as ContextType<'a, Key>>::Node>;
}

impl<'a, Ctx, Key> GraphType<'a, Ctx, Key> for FunGraph<'a, Ctx, Key>
where
    Key: Eq,
    Ctx: ContextType<'a, Key>,
{
    fn empty() -> Self {
        Self::Empty
    }
}

trait ContextType<'a, Key: Eq = Self> {
    type Node: 'a + HasKey<Key = Key>;
    type Edge: 'a + HasKey<Key = Key>;

    fn get_edges(&self) -> impl Iterator<Item = &'a Self::Edge>;
}

trait HasKey {
    type Key: Eq;

    fn key_ref(&self) -> &Self::Key;
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

    struct TestCtx<'a>(isize, &'a HashMap<isize, [isize; 4]>);

    impl<'a> HasKey for TestCtx<'a> {
        type Key = isize;

        fn key_ref(&self) -> &Self::Key {
            &self.0
        }
    }

    impl<'a> ContextType<'a, isize> for TestCtx<'a> {
        type Node = isize;
        type Edge = isize;

        fn get_edges(&self) -> impl Iterator<Item = &'a Self::Edge> {
            self.1
                .get(&self.0)
                .expect("node must exist")
                .iter()
                .filter_map(|i| if *i >= 0 { Some(i) } else { None })
        }
    }

    impl<T: Eq> HasKey for T {
        type Key = Self;

        fn key_ref(&self) -> &Self::Key {
            &self
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
        let ctx = TestCtx(key, &adj_list);
        let act: Vec<isize> = ctx.get_edges().map(|&i| i).collect();

        assert_eq!(act, exp);
    }

    impl<'a> GraphType<'a, isize, isize> for HashMap<isize, [isize; 4]> {
        fn empty() -> Self {
            HashMap::new()
        }
    }

    struct MapGraph<K, E, V>(Option<K>, HashMap<K, V>)
    where
        K: HasKey,
        E: HasKey,
        V: IntoIterator<Item = E>;

    impl<'a, K, E, V, C> GraphType<'a, C, <K as HasKey>::Key> for MapGraph<K, E, V>
    where
        K: HasKey,
        E: HasKey,
        V: IntoIterator<Item = E>,
        C: ContextType<'a, <E as HasKey>::Key>,
    {
        fn empty() -> Self {
            Self(None, HashMap::new())
        }
    }

    struct MapContext<'a>()

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
    fn can_iterate_graph_of_one_node(
        #[case] key: isize,
        #[case] exp: Vec<isize>,
        adj_list: HashMap<isize, [isize; 4]>,
    ) {
        let ctx = TestCtx(key, &adj_list);
        let graph = TestGraph::new(ctx);
        let act: Vec<isize> = graph.iter_dfs().map(|&i| i).collect();

        assert_eq!(exp, act);
    }

    // // 0 --- 1 --- 5 ---   (0, [1, 3, 2, -1]),
    // // | \_  |     | \_|   (1, [5, 3, 0, -1]),
    // // |   \ |    /        (2, [0, 3, 6, 4]),
    // // 2 --- 3   /         (3, [1, 6, 2, 0]),
    // // | \_  |  /          (4, [2, -1, -1, -1]),
    // // |   \ | /           (5, [5, 6, 1, -1]),
    // // 4     6             (6, [3, 5, 2, -1]),
    // #[rstest]
    // #[case(0,vec![0,1,5,6,3,2,4])]
    // #[case(1,vec![1,5,6,3,2,0,4])]
    // #[case(2,vec![2,0,1,5,6,3,4])]
    // #[case(3,vec![3,1,5,6,3,2,4])]
    // #[case(4,vec![4,2,0,1,5,6,3])]
    // #[case(5,vec![5,6,3,1,0,2,4])]
    // #[case(6,vec![6,3,1,5,0,2,4])]
    // fn can_iterate_using_dfs_order(
    //     #[case] key: isize,
    //     #[case] exp: Vec<isize>,
    //     adj_list: HashMap<isize, [isize; 4]>,
    // ) {
    //     let node = AdjNode(key, &adj_list);
    //     let act: Vec<isize> = node.iter_dfs().map(|adj| adj.0).collect();

    //     assert_eq!(exp, act);
    // }
}

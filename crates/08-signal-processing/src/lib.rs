use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::Reverse;

pub type NodeId = usize;

/// Directed weighted graph using adjacency lists.
/// For unweighted graphs, pass weight = 1.
pub struct Graph {
    adj: HashMap<NodeId, Vec<(NodeId, u64)>>,
    node_count: usize,
}

impl Graph {
    pub fn new(node_count: usize) -> Self { todo!() }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: u64) { todo!() }

    pub fn add_undirected_edge(&mut self, a: NodeId, b: NodeId, weight: u64) { todo!() }

    /// BFS from `start`. Returns nodes in visit order (level by level).
    pub fn bfs(&self, start: NodeId) -> Vec<NodeId> { todo!() }

    /// DFS from `start`. Returns nodes in visit order (depth first).
    pub fn dfs(&self, start: NodeId) -> Vec<NodeId> { todo!() }

    fn dfs_inner(&self, node: NodeId, visited: &mut HashSet<NodeId>, order: &mut Vec<NodeId>) { todo!() }

    /// Dijkstra's shortest paths from `start`.
    /// Returns a map of node -> minimum distance. Unreachable nodes are absent.
    pub fn dijkstra(&self, start: NodeId) -> HashMap<NodeId, u64> { todo!() }

    /// Topological sort via Kahn's algorithm (in-degree).
    /// Returns `None` if the graph contains a cycle.
    pub fn topological_sort(&self) -> Option<Vec<NodeId>> { todo!() }

    /// Returns true if the graph contains at least one cycle.
    pub fn has_cycle(&self) -> bool { todo!() }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod bfs {
        use super::*;

        #[test]
        fn visits_start_node_first() {
            let mut g = Graph::new(3);
            g.add_edge(0, 1, 1);
            g.add_edge(1, 2, 1);
            assert_eq!(g.bfs(0)[0], 0);
        }

        #[test]
        fn visits_all_reachable_nodes() {
            let mut g = Graph::new(4);
            g.add_edge(0, 1, 1);
            g.add_edge(0, 2, 1);
            g.add_edge(1, 3, 1);
            let order = g.bfs(0);
            assert_eq!(order.len(), 4);
        }

        #[test]
        fn level_order_before_depth() {
            let mut g = Graph::new(4);
            g.add_edge(0, 1, 1);
            g.add_edge(0, 2, 1);
            g.add_edge(1, 3, 1);
            g.add_edge(2, 3, 1);
            let order = g.bfs(0);
            assert_eq!(order[0], 0);
            assert_eq!(order[3], 3);
        }

        #[test]
        fn does_not_visit_unreachable_nodes() {
            let mut g = Graph::new(4);
            g.add_edge(0, 1, 1);
            let order = g.bfs(0);
            assert_eq!(order.len(), 2);
        }
    }

    mod dfs {
        use super::*;

        #[test]
        fn visits_deeply_before_widely() {
            let mut g = Graph::new(4);
            g.add_edge(0, 1, 1);
            g.add_edge(1, 2, 1);
            g.add_edge(0, 3, 1);
            let order = g.dfs(0);
            let pos1 = order.iter().position(|&n| n == 1).unwrap();
            let pos3 = order.iter().position(|&n| n == 3).unwrap();
            assert!(pos1 < pos3);
        }

        #[test]
        fn visits_all_reachable_nodes() {
            let mut g = Graph::new(3);
            g.add_edge(0, 1, 1);
            g.add_edge(1, 2, 1);
            assert_eq!(g.dfs(0).len(), 3);
        }
    }

    mod dijkstra {
        use super::*;

        #[test]
        fn distance_to_start_is_zero() {
            let g = Graph::new(3);
            assert_eq!(g.dijkstra(0)[&0], 0);
        }

        #[test]
        fn finds_shortest_path_over_direct_edge() {
            let mut g = Graph::new(3);
            g.add_edge(0, 1, 10);
            g.add_edge(0, 2, 1);
            g.add_edge(2, 1, 2);
            let dist = g.dijkstra(0);
            assert_eq!(dist[&1], 3);
            assert_eq!(dist[&2], 1);
        }

        #[test]
        fn unreachable_nodes_are_absent() {
            let mut g = Graph::new(3);
            g.add_edge(0, 1, 1);
            let dist = g.dijkstra(0);
            assert!(!dist.contains_key(&2));
        }

        #[test]
        fn single_node_graph() {
            let g = Graph::new(1);
            let dist = g.dijkstra(0);
            assert_eq!(dist[&0], 0);
            assert_eq!(dist.len(), 1);
        }
    }

    mod topological_sort {
        use super::*;

        #[test]
        fn respects_dependency_order() {
            let mut g = Graph::new(3);
            g.add_edge(0, 1, 1);
            g.add_edge(1, 2, 1);
            let order = g.topological_sort().unwrap();
            let pos = |n: usize| order.iter().position(|&x| x == n).unwrap();
            assert!(pos(0) < pos(1));
            assert!(pos(1) < pos(2));
        }

        #[test]
        fn detects_cycle() {
            let mut g = Graph::new(3);
            g.add_edge(0, 1, 1);
            g.add_edge(1, 2, 1);
            g.add_edge(2, 0, 1);
            assert!(g.topological_sort().is_none());
        }

        #[test]
        fn dag_with_multiple_valid_orderings_returns_some() {
            let mut g = Graph::new(4);
            g.add_edge(0, 2, 1);
            g.add_edge(1, 2, 1);
            g.add_edge(2, 3, 1);
            let order = g.topological_sort().unwrap();
            let pos = |n: usize| order.iter().position(|&x| x == n).unwrap();
            assert!(pos(0) < pos(2));
            assert!(pos(1) < pos(2));
            assert!(pos(2) < pos(3));
        }
    }
}

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
    pub fn new(node_count: usize) -> Self {
        let mut adj = HashMap::new();
        for i in 0..node_count {
            adj.insert(i, Vec::new());
        }
        Self { adj, node_count }
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: u64) {
        self.adj.entry(from).or_default().push((to, weight));
    }

    pub fn add_undirected_edge(&mut self, a: NodeId, b: NodeId, weight: u64) {
        self.add_edge(a, b, weight);
        self.add_edge(b, a, weight);
    }

    /// BFS from `start`. Returns nodes in visit order (level by level).
    pub fn bfs(&self, start: NodeId) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut order = Vec::new();
        visited.insert(start);
        queue.push_back(start);
        while let Some(node) = queue.pop_front() {
            order.push(node);
            let mut neighbours: Vec<_> = self.adj[&node].iter().map(|&(n, _)| n).collect();
            neighbours.sort_unstable(); // deterministic
            for next in neighbours {
                if visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        order
    }

    /// DFS from `start`. Returns nodes in visit order (depth first).
    pub fn dfs(&self, start: NodeId) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut order = Vec::new();
        self.dfs_inner(start, &mut visited, &mut order);
        order
    }

    fn dfs_inner(&self, node: NodeId, visited: &mut HashSet<NodeId>, order: &mut Vec<NodeId>) {
        if !visited.insert(node) {
            return;
        }
        order.push(node);
        let mut neighbours: Vec<_> = self.adj[&node].iter().map(|&(n, _)| n).collect();
        neighbours.sort_unstable();
        for next in neighbours {
            self.dfs_inner(next, visited, order);
        }
    }

    /// Dijkstra's shortest paths from `start`.
    /// Returns a map of node -> minimum distance. Unreachable nodes are absent.
    pub fn dijkstra(&self, start: NodeId) -> HashMap<NodeId, u64> {
        let mut dist: HashMap<NodeId, u64> = HashMap::new();
        let mut heap: BinaryHeap<Reverse<(u64, NodeId)>> = BinaryHeap::new();
        dist.insert(start, 0);
        heap.push(Reverse((0, start)));
        while let Some(Reverse((cost, node))) = heap.pop() {
            if cost > *dist.get(&node).unwrap_or(&u64::MAX) {
                continue;
            }
            for &(next, weight) in &self.adj[&node] {
                let next_cost = cost + weight;
                if next_cost < *dist.get(&next).unwrap_or(&u64::MAX) {
                    dist.insert(next, next_cost);
                    heap.push(Reverse((next_cost, next)));
                }
            }
        }
        dist
    }

    /// Topological sort via Kahn's algorithm (in-degree).
    /// Returns `None` if the graph contains a cycle.
    pub fn topological_sort(&self) -> Option<Vec<NodeId>> {
        let mut in_deg: HashMap<NodeId, usize> = (0..self.node_count).map(|n| (n, 0)).collect();
        for (&from, edges) in &self.adj {
            let _ = in_deg.entry(from).or_insert(0);
            for &(to, _) in edges {
                *in_deg.entry(to).or_insert(0) += 1;
            }
        }
        let mut queue: Vec<NodeId> = in_deg.iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&n, _)| n)
            .collect();
        queue.sort_unstable();
        let mut result = Vec::new();
        while !queue.is_empty() {
            queue.sort_unstable();
            let node = queue.remove(0);
            result.push(node);
            let mut neighbours: Vec<_> = self.adj[&node].iter().map(|&(n, _)| n).collect();
            neighbours.sort_unstable();
            for next in neighbours {
                let d = in_deg.get_mut(&next).unwrap();
                *d -= 1;
                if *d == 0 {
                    queue.push(next);
                }
            }
        }
        if result.len() == self.node_count { Some(result) } else { None }
    }

    /// Returns true if the graph contains at least one cycle.
    pub fn has_cycle(&self) -> bool {
        self.topological_sort().is_none()
    }
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
            //  0 -> 1 -> 3
            //  0 -> 2 -> 3
            let mut g = Graph::new(4);
            g.add_edge(0, 1, 1);
            g.add_edge(0, 2, 1);
            g.add_edge(1, 3, 1);
            g.add_edge(2, 3, 1);
            let order = g.bfs(0);
            // 0 first, 3 last (both paths lead to it)
            assert_eq!(order[0], 0);
            assert_eq!(order[3], 3);
        }

        #[test]
        fn does_not_visit_unreachable_nodes() {
            let mut g = Graph::new(4);
            g.add_edge(0, 1, 1);
            // node 2 and 3 are unreachable from 0
            let order = g.bfs(0);
            assert_eq!(order.len(), 2);
        }
    }

    mod dfs {
        use super::*;

        #[test]
        fn visits_deeply_before_widely() {
            //  0 -> 1 -> 2
            //  0 -> 3
            let mut g = Graph::new(4);
            g.add_edge(0, 1, 1);
            g.add_edge(1, 2, 1);
            g.add_edge(0, 3, 1);
            let order = g.dfs(0);
            // 1 should appear before 3 (depth first along 0->1->2)
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
            g.add_edge(2, 1, 2); // 0->2->1 costs 3, cheaper than direct 10
            let dist = g.dijkstra(0);
            assert_eq!(dist[&1], 3);
            assert_eq!(dist[&2], 1);
        }

        #[test]
        fn unreachable_nodes_are_absent() {
            let mut g = Graph::new(3);
            g.add_edge(0, 1, 1);
            // node 2 unreachable
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
            // a -> b -> c  (a must come before b, b before c)
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
            g.add_edge(2, 0, 1); // cycle
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

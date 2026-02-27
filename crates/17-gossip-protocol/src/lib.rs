// ============================================================
//  YOUR CHALLENGE — simulate an epidemic (gossip) protocol.
//
//  Gossip works by infection: in each round, every infected
//  node tells `fanout` randomly chosen neighbours its state.
//  Over rounds, information spreads like a disease.
//
//  Key properties:
//    - Convergence: eventually all nodes know the new state
//    - Resilience: works even with node failures
//    - O(log n) rounds to reach the whole cluster
//
//  Implement:
//    - Cluster::new(n_nodes, fanout) — n nodes, each gossips to `fanout` peers
//    - cluster.broadcast(origin, value) — node `origin` has new state `value`
//    - cluster.step(rng) — one round of gossip
//    - cluster.converged() — true if all nodes know the latest value
//    - cluster.round_count() — how many rounds have elapsed
// ============================================================

use rand::Rng;

#[derive(Clone, Debug)]
pub struct Node {
    pub id: usize,
    pub value: Option<u64>, // None = hasn't heard yet
}

pub struct Cluster {
    pub nodes: Vec<Node>,
    fanout: usize,
    rounds: usize,
    target_value: Option<u64>,
}

impl Cluster {
    pub fn new(n_nodes: usize, fanout: usize) -> Self {
        let nodes = (0..n_nodes).map(|id| Node { id, value: None }).collect();
        Self { nodes, fanout, rounds: 0, target_value: None }
    }

    /// Node `origin` receives a new value — starts the gossip cascade.
    pub fn broadcast(&mut self, origin: usize, value: u64) {
        self.target_value = Some(value);
        self.nodes[origin].value = Some(value);
    }

    /// One round: every informed node gossips to `fanout` random neighbours.
    pub fn step(&mut self, rng: &mut impl Rng) {
        let n = self.nodes.len();
        let target = self.target_value;
        // Collect ids of currently informed nodes
        let informed: Vec<usize> = self.nodes.iter()
            .filter(|node| node.value == target)
            .map(|node| node.id)
            .collect();

        for _src in informed {
            for _ in 0..self.fanout {
                let peer = rng.gen_range(0..n);
                self.nodes[peer].value = target;
            }
        }
        self.rounds += 1;
    }

    /// True if every node has the target value.
    pub fn converged(&self) -> bool {
        match self.target_value {
            None => false,
            Some(v) => self.nodes.iter().all(|n| n.value == Some(v)),
        }
    }

    pub fn round_count(&self) -> usize {
        self.rounds
    }

    /// Count of nodes that have received the value.
    pub fn informed_count(&self) -> usize {
        match self.target_value {
            None => 0,
            Some(v) => self.nodes.iter().filter(|n| n.value == Some(v)).count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    mod spreading {
        use super::*;

        #[test]
        fn single_node_cluster_converges_immediately() {
            let mut cluster = Cluster::new(1, 1);
            cluster.broadcast(0, 42);
            assert!(cluster.converged());
        }

        #[test]
        fn broadcast_marks_origin_as_informed() {
            let mut cluster = Cluster::new(10, 3);
            cluster.broadcast(0, 99);
            assert_eq!(cluster.informed_count(), 1);
        }

        #[test]
        fn gossip_reaches_all_nodes_within_log2n_times_2_rounds() {
            let n = 64;
            let fanout = 3;
            let max_rounds = ((n as f64).log2() * 2.0).ceil() as usize + 2;
            let mut cluster = Cluster::new(n, fanout);
            let mut rng = StdRng::seed_from_u64(42);
            cluster.broadcast(0, 1);
            while !cluster.converged() && cluster.round_count() < max_rounds {
                cluster.step(&mut rng);
            }
            assert!(cluster.converged(),
                "did not converge in {max_rounds} rounds (informed: {}/{})",
                cluster.informed_count(), n);
        }

        #[test]
        fn informed_count_grows_monotonically() {
            let mut cluster = Cluster::new(20, 2);
            let mut rng = StdRng::seed_from_u64(7);
            cluster.broadcast(0, 1);
            let mut prev = cluster.informed_count();
            for _ in 0..10 {
                cluster.step(&mut rng);
                let curr = cluster.informed_count();
                assert!(curr >= prev, "informed count went backwards: {prev} → {curr}");
                prev = curr;
            }
        }

        #[test]
        fn higher_fanout_converges_faster() {
            let n = 100;
            let mut rng_lo = StdRng::seed_from_u64(42);
            let mut rng_hi = StdRng::seed_from_u64(42);

            let mut low  = Cluster::new(n, 1);
            let mut high = Cluster::new(n, 5);
            low.broadcast(0, 1);
            high.broadcast(0, 1);

            while !low.converged()  { low.step(&mut rng_lo); }
            while !high.converged() { high.step(&mut rng_hi); }

            assert!(high.round_count() < low.round_count(),
                "high fanout ({} rounds) should be faster than low ({} rounds)",
                high.round_count(), low.round_count());
        }
    }
}

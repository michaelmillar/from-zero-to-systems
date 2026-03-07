// ============================================================
//  YOUR CHALLENGE - implement a simplified Raft consensus node.
//
//  Raft has three roles: Follower, Candidate, Leader.
//  This simulation runs Raft in a single-threaded, in-memory
//  cluster - no networking, no async.
//
//  Core concepts to implement:
//    - Term: logical clock that monotonically increases
//    - Leader election: the node with the most votes wins
//    - Log replication: leader appends entries; followers accept
//    - Commit: entry is committed when a majority acknowledges it
//
//  API (simplified):
//    RaftCluster::new(n) - create n nodes, all start as followers
//    cluster.tick() - run one election + replication round
//    cluster.leader() -> Option<usize> - which node is leader?
//    cluster.append(data: &str) -> bool - leader appends an entry
//    cluster.committed_log() -> Vec<String> - entries committed by majority
//
//  Hint: in this deterministic simulation, node 0 always starts
//  the election. It increments its term, votes for itself, then
//  requests votes from all other nodes still on the old term.
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Role { Follower, Candidate, Leader }

#[derive(Debug, Clone)]
pub struct LogEntry { pub term: u64, pub data: String }

#[derive(Debug, Clone)]
pub struct RaftNode {
    pub id: usize,
    pub role: Role,
    pub current_term: u64,
    pub voted_for: Option<usize>,
    pub log: Vec<LogEntry>,
    pub commit_index: usize,
    pub votes_received: usize,
}

pub struct RaftCluster {
    pub nodes: Vec<RaftNode>,
    n: usize,
}

impl RaftCluster {
    /// Create `n` Raft nodes, all starting as Followers in term 0.
    pub fn new(n: usize) -> Self {
        let nodes = (0..n).map(|id| RaftNode {
            id,
            role: Role::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            votes_received: 0,
        }).collect();
        Self { nodes, n }
    }

    /// Simulate one election round.
    pub fn tick(&mut self) {
        todo!()
    }

    /// Index of the current Leader, if one exists.
    pub fn leader(&self) -> Option<usize> {
        todo!()
    }

    /// Leader appends a new entry to its log and replicates to followers.
    /// Returns false if there is no leader.
    pub fn append(&mut self, data: &str) -> bool {
        todo!()
    }

    /// Entries present in the logs of a majority of nodes (committed).
    pub fn committed_log(&self) -> Vec<String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod election {
        use super::*;

        #[test]
        fn cluster_of_one_elects_itself_immediately() {
            let mut cluster = RaftCluster::new(1);
            cluster.tick();
            assert_eq!(cluster.leader(), Some(0));
        }

        #[test]
        fn three_node_cluster_elects_a_leader_after_tick() {
            let mut cluster = RaftCluster::new(3);
            cluster.tick();
            assert!(cluster.leader().is_some(), "should have a leader after tick");
        }

        #[test]
        fn elected_node_has_role_leader() {
            let mut cluster = RaftCluster::new(5);
            cluster.tick();
            let leader_id = cluster.leader().unwrap();
            assert_eq!(cluster.nodes[leader_id].role, Role::Leader);
        }

        #[test]
        fn only_one_leader_exists_after_election() {
            let mut cluster = RaftCluster::new(5);
            cluster.tick();
            let leader_count = cluster.nodes.iter().filter(|n| n.role == Role::Leader).count();
            assert_eq!(leader_count, 1);
        }

        #[test]
        fn all_followers_have_updated_term() {
            let mut cluster = RaftCluster::new(3);
            cluster.tick();
            let leader_term = cluster.nodes[cluster.leader().unwrap()].current_term;
            for node in &cluster.nodes {
                if node.role != Role::Leader {
                    assert_eq!(node.current_term, leader_term,
                        "follower {} has stale term", node.id);
                }
            }
        }
    }

    mod log_replication {
        use super::*;

        #[test]
        fn appending_to_leader_returns_true() {
            let mut cluster = RaftCluster::new(3);
            cluster.tick();
            assert!(cluster.append("first entry"));
        }

        #[test]
        fn appending_without_leader_returns_false() {
            let mut cluster = RaftCluster::new(3);
            // No tick - no leader elected yet
            assert!(!cluster.append("no leader"));
        }

        #[test]
        fn committed_log_contains_appended_entries_in_order() {
            let mut cluster = RaftCluster::new(3);
            cluster.tick();
            cluster.append("entry-a");
            cluster.append("entry-b");
            cluster.append("entry-c");
            let log = cluster.committed_log();
            assert_eq!(log, vec!["entry-a", "entry-b", "entry-c"]);
        }

        #[test]
        fn entry_is_replicated_to_majority_of_nodes() {
            let mut cluster = RaftCluster::new(5);
            cluster.tick();
            cluster.append("replicated");
            // At least 3 of 5 nodes (majority) should have this entry
            let count = cluster.nodes.iter()
                .filter(|n| n.log.iter().any(|e| e.data == "replicated"))
                .count();
            assert!(count >= 3, "only {count}/5 nodes have the entry");
        }
    }
}

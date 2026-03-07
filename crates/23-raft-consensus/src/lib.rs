// ============================================================
//  YOUR CHALLENGE — implement a simplified Raft consensus node.
//
//  Raft has three roles: Follower, Candidate, Leader.
//  This simulation runs Raft in a single-threaded, in-memory
//  cluster — no networking, no async.
//
//  Core concepts to implement:
//    - Term: logical clock that monotonically increases
//    - Leader election: the node with the most votes wins
//    - Log replication: leader appends entries; followers accept
//    - Commit: entry is committed when a majority acknowledges it
//
//  API (simplified):
//    RaftCluster::new(n) — create n nodes, all start as followers
//    cluster.tick() — run one election + replication round
//    cluster.leader() -> Option<usize> — which node is leader?
//    cluster.append(data: &str) -> bool — leader appends an entry
//    cluster.committed_log() -> Vec<String> — entries committed by majority
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
        // If a leader already exists, nothing to do.
        if self.leader().is_some() { return; }

        let majority = self.n / 2 + 1;
        let new_term = self.nodes.iter().map(|nd| nd.current_term).max().unwrap_or(0) + 1;

        // In this deterministic simulation, node 0 starts the election
        // (in real Raft this would be the first node whose timeout fires).
        let candidate_id = 0;

        // Candidate increments its term and votes for itself.
        self.nodes[candidate_id].role = Role::Candidate;
        self.nodes[candidate_id].current_term = new_term;
        self.nodes[candidate_id].voted_for = Some(candidate_id);
        let mut votes = 1usize;

        // Request votes from all other nodes (still in the old term, unvoted).
        for i in 1..self.n {
            let voter = &mut self.nodes[i];
            if new_term > voter.current_term && voter.voted_for.is_none() {
                voter.current_term = new_term;
                voter.voted_for = Some(candidate_id);
                votes += 1;
            }
        }

        self.nodes[candidate_id].votes_received = votes;

        // Declare winner if majority reached.
        if votes >= majority {
            self.nodes[candidate_id].role = Role::Leader;
            for i in 1..self.n {
                self.nodes[i].role = Role::Follower;
                self.nodes[i].current_term = new_term; // bring all to same term
            }
        }
    }

    /// Index of the current Leader, if one exists.
    pub fn leader(&self) -> Option<usize> {
        self.nodes.iter().find(|n| n.role == Role::Leader).map(|n| n.id)
    }

    /// Leader appends a new entry to its log and replicates to followers.
    /// Returns false if there is no leader.
    pub fn append(&mut self, data: &str) -> bool {
        let leader_id = match self.leader() {
            Some(id) => id,
            None => return false,
        };
        let term = self.nodes[leader_id].current_term;
        let entry = LogEntry { term, data: data.to_string() };

        // Append to leader's log.
        self.nodes[leader_id].log.push(entry.clone());

        // Replicate to all followers immediately (synchronous in this simulation).
        for node in &mut self.nodes {
            if node.id != leader_id {
                node.log.push(entry.clone());
            }
        }

        true
    }

    /// Entries present in the logs of a majority of nodes (committed).
    pub fn committed_log(&self) -> Vec<String> {
        let majority = self.n / 2 + 1;
        // Find the length of the shortest log held by a majority.
        // Since we replicate synchronously, all logs are the same length,
        // but we implement the general majority-overlap check for correctness.
        if self.nodes.is_empty() { return vec![]; }
        let min_len = self.nodes.iter().map(|n| n.log.len()).min().unwrap_or(0);
        // An entry at position i is committed if ≥ majority nodes have it.
        let mut result = Vec::new();
        for i in 0..min_len {
            let count = self.nodes.iter().filter(|n| n.log.len() > i).count();
            if count >= majority {
                result.push(self.nodes[0].log[i].data.clone());
            } else {
                break;
            }
        }
        result
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
            // No tick — no leader elected yet
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

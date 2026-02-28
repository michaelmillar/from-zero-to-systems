// ============================================================
//  YOUR CHALLENGE — implement a consistent hash ring.
//
//  A consistent hash ring places both nodes and keys on a
//  circular hash space [0, 2^64). To find which node owns
//  a key, hash the key and walk clockwise to the next node.
//
//  Virtual nodes: each physical node gets `vnodes` slots on
//  the ring (e.g. "node-a#0", "node-a#1", ...). This spreads
//  load more evenly when nodes are added or removed.
//
//  API:
//    HashRing::new(vnodes: usize) -> Self
//    ring.add_node(name: &str)
//    ring.remove_node(name: &str)
//    ring.get_node(key: &str) -> Option<&str>   // owner of `key`
//    ring.node_count() -> usize                  // physical nodes
// ============================================================

use std::collections::BTreeMap;

pub struct HashRing {
    vnodes: usize,
    ring: BTreeMap<u64, String>, // hash → physical node name
}

impl HashRing {
    pub fn new(vnodes: usize) -> Self {
        Self { vnodes, ring: BTreeMap::new() }
    }

    pub fn add_node(&mut self, name: &str) {
        self.remove_node(name);
        let base = fnv1a(name);
        for i in 0..self.vnodes {
            let key = mmh3_mix(base ^ (i as u64));
            self.ring.insert(key, name.to_string());
        }
    }

    pub fn remove_node(&mut self, name: &str) {
        let base = fnv1a(name);
        for i in 0..self.vnodes {
            let key = mmh3_mix(base ^ (i as u64));
            self.ring.remove(&key);
        }
    }

    pub fn get_node(&self, key: &str) -> Option<&str> {
        if self.ring.is_empty() { return None; }
        let hash = fnv1a(key);
        // Walk clockwise: find first node at or after hash; wrap around if needed
        self.ring.range(hash..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, v)| v.as_str())
    }

    pub fn node_count(&self) -> usize {
        self.ring.values()
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
}

/// MurmurHash3 64-bit finaliser — excellent avalanche for vnode placement.
pub fn mmh3_mix(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}

/// FNV-1a 64-bit hash — fast, good distribution, no external deps.
pub fn fnv1a(data: &str) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for byte in data.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ring_setup {
        use super::*;

        #[test]
        fn empty_ring_returns_none_for_any_key() {
            let ring = HashRing::new(10);
            assert!(ring.get_node("some-key").is_none());
        }

        #[test]
        fn single_node_owns_all_keys() {
            let mut ring = HashRing::new(10);
            ring.add_node("alpha");
            assert_eq!(ring.get_node("any-key"), Some("alpha"));
            assert_eq!(ring.get_node("other-key"), Some("alpha"));
        }

        #[test]
        fn node_count_tracks_physical_nodes() {
            let mut ring = HashRing::new(10);
            ring.add_node("alpha");
            ring.add_node("beta");
            assert_eq!(ring.node_count(), 2);
        }

        #[test]
        fn adding_same_node_twice_does_not_duplicate() {
            let mut ring = HashRing::new(10);
            ring.add_node("alpha");
            ring.add_node("alpha");
            assert_eq!(ring.node_count(), 1);
        }
    }

    mod routing {
        use super::*;

        #[test]
        fn key_always_routes_to_same_node_deterministically() {
            let mut ring = HashRing::new(100);
            ring.add_node("alpha");
            ring.add_node("beta");
            ring.add_node("gamma");
            let first  = ring.get_node("user:42").unwrap().to_string();
            let second = ring.get_node("user:42").unwrap().to_string();
            assert_eq!(first, second);
        }

        #[test]
        fn keys_distribute_across_multiple_nodes() {
            let mut ring = HashRing::new(150);
            ring.add_node("alpha");
            ring.add_node("beta");
            ring.add_node("gamma");
            let keys: Vec<String> = (0..3000).map(|i| format!("key:{i}")).collect();
            let nodes: std::collections::HashSet<String> = keys.iter()
                .filter_map(|k| ring.get_node(k).map(str::to_string))
                .collect();
            assert_eq!(nodes.len(), 3, "all three nodes should own at least one key");
        }

        #[test]
        fn removing_a_node_reassigns_its_keys_to_remaining_nodes() {
            let mut ring = HashRing::new(100);
            ring.add_node("alpha");
            ring.add_node("beta");
            ring.add_node("gamma");

            // Find a key owned by beta
            let key = (0..1000)
                .map(|i| format!("k{i}"))
                .find(|k| ring.get_node(k) == Some("beta"))
                .expect("beta should own at least one of 1000 keys");

            ring.remove_node("beta");

            // After removal, key routes to alpha or gamma, not beta
            let new_owner = ring.get_node(&key).unwrap();
            assert_ne!(new_owner, "beta");
        }

        #[test]
        fn removing_all_nodes_returns_none() {
            let mut ring = HashRing::new(10);
            ring.add_node("alpha");
            ring.remove_node("alpha");
            assert!(ring.get_node("key").is_none());
        }
    }

    mod vnodes {
        use super::*;

        #[test]
        fn more_vnodes_produces_more_even_distribution() {
            // With 3 nodes and high vnodes, each should own roughly 1/3 of keys
            let mut ring = HashRing::new(200);
            ring.add_node("alpha");
            ring.add_node("beta");
            ring.add_node("gamma");

            let n = 3000_usize;
            let mut counts = std::collections::HashMap::new();
            for i in 0..n {
                let owner = ring.get_node(&format!("key:{i}")).unwrap();
                *counts.entry(owner.to_string()).or_insert(0usize) += 1;
            }
            // Each node should own between 20% and 47% of keys
            for (_node, count) in &counts {
                let fraction = *count as f64 / n as f64;
                assert!(fraction > 0.20 && fraction < 0.47,
                    "node owns {:.1}% — too skewed", fraction * 100.0);
            }
        }
    }
}

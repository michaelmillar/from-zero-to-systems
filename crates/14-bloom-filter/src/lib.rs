// ============================================================
//  YOUR CHALLENGE - implement a Bloom filter.
//
//  A Bloom filter is a probabilistic data structure that:
//    - NEVER has false negatives: if you inserted X, contains(X) = true
//    - MAY have false positives: contains(Y) can be true even if Y was never inserted
//
//  Implementation:
//    - A bit array of `m` bits
//    - `k` independent hash functions (simulate with FNV-1a + different seeds)
//    - insert: set bit[hash_i(item) % m] for each i in 0..k
//    - contains: check bit[hash_i(item) % m] for each i; true iff ALL are set
//
//  The optimal false positive rate for n items in m bits with k hashes is:
//    p approximately (1 - e^(-kn/m))^k
//
//  For this exercise, use k=3 hash functions with seeds 0, 1, 2.
//  hash_with_seed is already implemented below - use it.
// ============================================================

pub struct BloomFilter {
    bits: Vec<bool>,
    k: usize,      // number of hash functions
    n_inserted: usize,
}

impl BloomFilter {
    /// Create a new Bloom filter with `m` bits and `k` hash functions.
    pub fn new(m: usize, k: usize) -> Self {
        Self { bits: vec![false; m], k, n_inserted: 0 }
    }

    /// Insert an item into the filter.
    pub fn insert(&mut self, item: &str) {
        todo!()
    }

    /// Check if an item might be in the filter.
    /// Returns false -> definitely not present.
    /// Returns true  -> probably present (may be a false positive).
    pub fn contains(&self, item: &str) -> bool {
        todo!()
    }

    /// Number of items inserted.
    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Estimated false positive probability given current fill level.
    pub fn estimated_fpr(&self) -> f64 {
        todo!()
    }
}

/// Hash `item` with a given `seed` to produce a bit index in [0, m).
/// Uses FNV-1a with the seed XORed into the initial state.
pub fn hash_with_seed(item: &str, seed: u64, m: usize) -> usize {
    let mut h: u64 = 14695981039346656037u64 ^ seed.wrapping_mul(1099511628211);
    for byte in item.bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(1099511628211);
    }
    (h % m as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    mod membership {
        use super::*;

        #[test]
        fn inserted_item_is_always_found() {
            let mut bf = BloomFilter::new(1024, 3);
            bf.insert("hello");
            assert!(bf.contains("hello"));
        }

        #[test]
        fn multiple_inserted_items_are_all_found() {
            let mut bf = BloomFilter::new(4096, 3);
            let words = ["alpha", "beta", "gamma", "delta", "epsilon"];
            for w in &words { bf.insert(w); }
            for w in &words {
                assert!(bf.contains(w), "{w} should be found after insertion");
            }
        }

        #[test]
        fn clearly_absent_item_is_probably_not_found() {
            // With a large filter and few items, false positives should be very rare
            let mut bf = BloomFilter::new(65536, 3);
            for i in 0..100 { bf.insert(&format!("item:{i}")); }
            // Test 1000 items that were never inserted
            let false_positives = (1000..2000)
                .filter(|i| bf.contains(&format!("item:{i}")))
                .count();
            assert!(false_positives < 50,
                "{false_positives} false positives - filter too lossy");
        }

        #[test]
        fn no_false_negatives_for_large_insertion_batch() {
            let mut bf = BloomFilter::new(16384, 3);
            for i in 0..500 { bf.insert(&format!("key:{i}")); }
            for i in 0..500 {
                assert!(bf.contains(&format!("key:{i}")), "false negative at {i}");
            }
        }
    }

    mod bookkeeping {
        use super::*;

        #[test]
        fn len_tracks_insertions() {
            let mut bf = BloomFilter::new(1024, 3);
            assert_eq!(bf.len(), 0);
            bf.insert("a");
            bf.insert("b");
            assert_eq!(bf.len(), 2);
        }

        #[test]
        fn estimated_fpr_increases_as_filter_fills() {
            let mut bf = BloomFilter::new(1024, 3);
            let fpr_empty = bf.estimated_fpr();
            for i in 0..200 { bf.insert(&format!("item:{i}")); }
            let fpr_full = bf.estimated_fpr();
            assert!(fpr_full > fpr_empty,
                "FPR should increase as filter fills: {fpr_empty:.4} -> {fpr_full:.4}");
        }
    }
}

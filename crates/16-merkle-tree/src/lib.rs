// ============================================================
//  YOUR CHALLENGE - implement a Merkle tree.
//
//  A Merkle tree is a binary tree where every leaf contains
//  the hash of a data block, and every internal node contains
//  the hash of its children concatenated:
//    parent_hash = hash(left_hash || right_hash)
//
//  Properties:
//    - The root hash summarises ALL data
//    - Changing any leaf changes every ancestor up to the root
//    - Membership proofs are O(log n) - you only need the siblings
//
//  Implementation notes:
//    - Use fnv1a for hashing (already implemented below)
//    - If there's an odd number of leaves, duplicate the last one
//    - A "proof" is a Vec<(u64, Side)> - sibling hash + which side it's on
//    - Verification: hash the leaf, then combine with each proof element
//    - combine(left, right) is already implemented below
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Side { Left, Right }

pub struct MerkleTree {
    /// All nodes, stored level-by-level: leaves first, root last.
    /// levels[0] = leaf hashes, levels.last() = [root]
    levels: Vec<Vec<u64>>,
}

impl MerkleTree {
    /// Build a Merkle tree from raw data blocks.
    pub fn build(data: &[&str]) -> Self {
        todo!()
    }

    /// Root hash of the tree.
    pub fn root(&self) -> Option<u64> {
        todo!()
    }

    /// Generate an inclusion proof for the leaf at `index`.
    /// Returns None if index is out of range.
    pub fn proof(&self, index: usize) -> Option<Vec<(u64, Side)>> {
        todo!()
    }

    /// Verify that `data` is at position `index` using the given proof and root.
    pub fn verify(data: &str, index: usize, proof: &[(u64, Side)], root: u64) -> bool {
        todo!()
    }
}

pub fn fnv1a(data: &str) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for b in data.bytes() { h ^= b as u64; h = h.wrapping_mul(1099511628211); }
    h
}

pub fn combine(left: u64, right: u64) -> u64 {
    fnv1a(&format!("{left:016x}{right:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod building {
        use super::*;

        #[test]
        fn single_leaf_root_is_its_hash() {
            let tree = MerkleTree::build(&["hello"]);
            assert_eq!(tree.root(), Some(fnv1a("hello")));
        }

        #[test]
        fn same_data_produces_same_root() {
            let a = MerkleTree::build(&["a", "b", "c", "d"]);
            let b = MerkleTree::build(&["a", "b", "c", "d"]);
            assert_eq!(a.root(), b.root());
        }

        #[test]
        fn different_data_produces_different_root() {
            let a = MerkleTree::build(&["a", "b", "c", "d"]);
            let b = MerkleTree::build(&["a", "b", "c", "X"]);
            assert_ne!(a.root(), b.root());
        }

        #[test]
        fn odd_number_of_leaves_is_handled() {
            // Should not panic - odd leaf is duplicated
            let tree = MerkleTree::build(&["a", "b", "c"]);
            assert!(tree.root().is_some());
        }

        #[test]
        fn empty_tree_has_no_root() {
            let tree = MerkleTree::build(&[]);
            assert!(tree.root().is_none());
        }
    }

    mod proofs {
        use super::*;

        #[test]
        fn proof_verifies_for_every_leaf() {
            let data = vec!["tx1", "tx2", "tx3", "tx4"];
            let tree = MerkleTree::build(&data);
            let root = tree.root().unwrap();
            for (i, &item) in data.iter().enumerate() {
                let proof = tree.proof(i).unwrap();
                assert!(
                    MerkleTree::verify(item, i, &proof, root),
                    "proof failed for leaf {i}: {item}"
                );
            }
        }

        #[test]
        fn tampered_data_fails_verification() {
            let data = vec!["tx1", "tx2", "tx3", "tx4"];
            let tree = MerkleTree::build(&data);
            let root = tree.root().unwrap();
            let proof = tree.proof(0).unwrap();
            assert!(!MerkleTree::verify("TAMPERED", 0, &proof, root));
        }

        #[test]
        fn proof_for_out_of_range_index_is_none() {
            let tree = MerkleTree::build(&["a", "b"]);
            assert!(tree.proof(99).is_none());
        }

        #[test]
        fn wrong_root_fails_verification() {
            let data = vec!["a", "b", "c", "d"];
            let tree = MerkleTree::build(&data);
            let proof = tree.proof(0).unwrap();
            assert!(!MerkleTree::verify("a", 0, &proof, 0xDEADBEEF));
        }
    }
}

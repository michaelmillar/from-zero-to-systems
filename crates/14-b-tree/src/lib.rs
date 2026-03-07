const ORDER: usize = 5; // max children per internal node
const MAX_KEYS: usize = ORDER - 1; // 4
const MIN_KEYS: usize = ORDER / 2; // 2 (min keys in non-root node)

#[derive(Debug, Clone)]
enum Node<K, V> {
    Leaf {
        keys: Vec<K>,
        values: Vec<V>,
    },
    Internal {
        keys: Vec<K>,        // separator keys; len = children.len() - 1
        children: Vec<Box<Node<K, V>>>,
    },
}

pub struct BTree<K: Ord + Clone, V: Clone> {
    root: Box<Node<K, V>>,
    len: usize,
}

impl<K: Ord + Clone, V: Clone> BTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: Box::new(Node::Leaf { keys: vec![], values: vec![] }),
            len: 0,
        }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn get(&self, key: &K) -> Option<&V> {
        Self::get_node(&self.root, key)
    }

    fn get_node<'a>(node: &'a Node<K, V>, key: &K) -> Option<&'a V> {
        match node {
            Node::Leaf { keys, values } => {
                keys.binary_search(key).ok().map(|i| &values[i])
            }
            Node::Internal { keys, children } => {
                let idx = keys.partition_point(|k| k <= key);
                // If exact match on separator, go right child
                let child_idx = if idx < keys.len() && &keys[idx] == key {
                    idx + 1
                } else {
                    idx
                };
                Self::get_node(&children[child_idx], key)
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(median) = Self::insert_node(&mut self.root, key, value) {
            // Root split -- create new root
            let old_root = std::mem::replace(
                &mut self.root,
                Box::new(Node::Internal { keys: vec![], children: vec![] }),
            );
            if let Node::Internal { keys, children } = self.root.as_mut() {
                keys.push(median.0);
                children.push(old_root);
                children.push(median.1);
            }
        }
        self.len += 1;
    }

    /// Returns Some((median_key, right_child)) if the node was split, None otherwise.
    fn insert_node(node: &mut Node<K, V>, key: K, value: V) -> Option<(K, Box<Node<K, V>>)> {
        match node {
            Node::Leaf { keys, values } => {
                let pos = keys.partition_point(|k| k < &key);
                keys.insert(pos, key);
                values.insert(pos, value);
                if keys.len() > MAX_KEYS {
                    Some(Self::split_leaf(keys, values))
                } else {
                    None
                }
            }
            Node::Internal { keys, children } => {
                let idx = keys.partition_point(|k| k <= &key);
                let child_idx = if idx < keys.len() && &keys[idx] == &key { idx + 1 } else { idx };
                if let Some((med_key, right)) = Self::insert_node(&mut children[child_idx], key, value) {
                    keys.insert(child_idx, med_key);
                    children.insert(child_idx + 1, right);
                    if keys.len() > MAX_KEYS {
                        Some(Self::split_internal(keys, children))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn split_leaf(keys: &mut Vec<K>, values: &mut Vec<V>) -> (K, Box<Node<K, V>>) {
        let mid = keys.len() / 2;
        let right_keys = keys.split_off(mid);
        let right_vals = values.split_off(mid);
        let median = right_keys[0].clone();
        (median, Box::new(Node::Leaf { keys: right_keys, values: right_vals }))
    }

    fn split_internal(
        keys: &mut Vec<K>,
        children: &mut Vec<Box<Node<K, V>>>,
    ) -> (K, Box<Node<K, V>>) {
        let mid = keys.len() / 2;
        let median = keys.remove(mid);
        let right_keys = keys.split_off(mid);
        let right_children = children.split_off(mid + 1);
        (median, Box::new(Node::Internal { keys: right_keys, children: right_children }))
    }

    /// Range scan: returns all (key, value) pairs where from <= key <= to.
    pub fn range(&self, from: &K, to: &K) -> Vec<(K, V)> {
        let mut out = Vec::new();
        Self::range_node(&self.root, from, to, &mut out);
        out
    }

    fn range_node(node: &Node<K, V>, from: &K, to: &K, out: &mut Vec<(K, V)>) {
        match node {
            Node::Leaf { keys, values } => {
                for (k, v) in keys.iter().zip(values.iter()) {
                    if k >= from && k <= to {
                        out.push((k.clone(), v.clone()));
                    }
                }
            }
            Node::Internal { keys, children } => {
                for (i, child) in children.iter().enumerate() {
                    // Prune: skip children whose key range can't overlap [from, to]
                    let lower_ok = i == 0 || &keys[i - 1] <= to;
                    let upper_ok = i == keys.len() || &keys[i] >= from;
                    if lower_ok && upper_ok {
                        Self::range_node(child, from, to, out);
                    }
                }
            }
        }
    }

    /// Returns all keys in sorted order (in-order traversal).
    pub fn keys(&self) -> Vec<K> {
        let mut out = Vec::new();
        Self::collect_keys(&self.root, &mut out);
        out
    }

    fn collect_keys(node: &Node<K, V>, out: &mut Vec<K>) {
        match node {
            Node::Leaf { keys, .. } => out.extend(keys.iter().cloned()),
            Node::Internal { children, .. } => {
                for child in children {
                    Self::collect_keys(child, out);
                }
            }
        }
    }
}

// Suppress dead_code warning for MIN_KEYS which documents the invariant
const _: () = assert!(MIN_KEYS == 2);

impl<K: Ord + Clone, V: Clone> Default for BTree<K, V> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod lookup {
        use super::*;

        #[test]
        fn get_on_empty_tree_returns_none() {
            let t: BTree<i32, &str> = BTree::new();
            assert_eq!(t.get(&1), None);
        }

        #[test]
        fn insert_then_get_returns_value() {
            let mut t = BTree::new();
            t.insert(10, "ten");
            assert_eq!(t.get(&10), Some(&"ten"));
        }

        #[test]
        fn missing_key_returns_none() {
            let mut t = BTree::new();
            t.insert(1, "one");
            assert_eq!(t.get(&2), None);
        }
    }

    mod ordering {
        use super::*;

        #[test]
        fn keys_are_always_sorted() {
            let mut t = BTree::new();
            for k in [5, 3, 8, 1, 9, 2, 7, 4, 6] {
                t.insert(k, k * 10);
            }
            let keys = t.keys();
            let sorted: Vec<i32> = (1..=9).collect();
            assert_eq!(keys, sorted);
        }

        #[test]
        fn reverse_insertion_order_produces_sorted_keys() {
            let mut t = BTree::new();
            for k in (1..=20).rev() {
                t.insert(k, ());
            }
            assert_eq!(t.keys(), (1..=20).collect::<Vec<_>>());
        }
    }

    mod splitting {
        use super::*;

        #[test]
        fn many_insertions_all_retrievable() {
            let mut t = BTree::new();
            for i in 0..100i32 {
                t.insert(i, i * 2);
            }
            for i in 0..100i32 {
                assert_eq!(t.get(&i), Some(&(i * 2)), "missing key {i}");
            }
        }

        #[test]
        fn len_is_correct_after_many_insertions() {
            let mut t: BTree<i32, ()> = BTree::new();
            for i in 0..50 {
                t.insert(i, ());
            }
            assert_eq!(t.len(), 50);
        }
    }

    mod range {
        use super::*;

        #[test]
        fn range_scan_returns_correct_subset() {
            let mut t = BTree::new();
            for i in 1..=10i32 {
                t.insert(i, i * 10);
            }
            let result = t.range(&3, &7);
            let keys: Vec<i32> = result.iter().map(|(k, _)| *k).collect();
            assert_eq!(keys, vec![3, 4, 5, 6, 7]);
        }

        #[test]
        fn range_with_no_matches_returns_empty() {
            let mut t: BTree<i32, ()> = BTree::new();
            for i in 1..=5 { t.insert(i, ()); }
            assert!(t.range(&10, &20).is_empty());
        }
    }
}

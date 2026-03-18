const ORDER: usize = 5;
const MAX_KEYS: usize = ORDER - 1; // 4
const MIN_KEYS: usize = ORDER / 2; // 2

#[derive(Debug, Clone)]
enum Node<K, V> {
    Leaf {
        keys: Vec<K>,
        values: Vec<V>,
    },
    Internal {
        keys: Vec<K>,
        children: Vec<Box<Node<K, V>>>,
    },
}

pub struct BTree<K: Ord + Clone, V: Clone> {
    root: Box<Node<K, V>>,
    len: usize,
}

impl<K: Ord + Clone, V: Clone> BTree<K, V> {
    pub fn new() -> Self { todo!() }

    pub fn len(&self) -> usize { todo!() }
    pub fn is_empty(&self) -> bool { todo!() }

    pub fn get(&self, key: &K) -> Option<&V> { todo!() }

    fn get_node<'a>(node: &'a Node<K, V>, key: &K) -> Option<&'a V> { todo!() }

    pub fn insert(&mut self, key: K, value: V) { todo!() }

    /// Returns Some((median_key, right_child)) if the node was split, None otherwise.
    fn insert_node(node: &mut Node<K, V>, key: K, value: V) -> Option<(K, Box<Node<K, V>>)> { todo!() }

    fn split_leaf(keys: &mut Vec<K>, values: &mut Vec<V>) -> (K, Box<Node<K, V>>) { todo!() }

    fn split_internal(
        keys: &mut Vec<K>,
        children: &mut Vec<Box<Node<K, V>>>,
    ) -> (K, Box<Node<K, V>>) { todo!() }

    /// Range scan: returns all (key, value) pairs where from <= key <= to.
    pub fn range(&self, from: &K, to: &K) -> Vec<(K, V)> { todo!() }

    fn range_node(node: &Node<K, V>, from: &K, to: &K, out: &mut Vec<(K, V)>) { todo!() }

    /// Returns all keys in sorted order (in-order traversal).
    pub fn keys(&self) -> Vec<K> { todo!() }

    fn collect_keys(node: &Node<K, V>, out: &mut Vec<K>) { todo!() }
}

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

const MAX_LEVEL: usize = 16;

struct Node<K, V> {
    key: K,
    value: V,
    /// next[i] is a raw pointer to the next node at level i.
    /// Ownership of all nodes flows through the level-0 chain only.
    next: Vec<*mut Node<K, V>>,
}

pub struct SkipList<K: Ord, V> {
    head: Box<Node<K, V>>,
    level: usize,
    len: usize,
}

unsafe impl<K: Ord + Send, V: Send> Send for SkipList<K, V> {}

/// Safely produce a `&Node` from a non-null raw pointer.
#[inline(always)]
unsafe fn node_ref<'a, K, V>(ptr: *mut Node<K, V>) -> &'a Node<K, V> {
    &*ptr
}

impl<K: Ord + Clone + Default, V: Clone + Default> SkipList<K, V> {
    pub fn new() -> Self { todo!() }

    pub fn len(&self) -> usize { todo!() }
    pub fn is_empty(&self) -> bool { todo!() }

    fn random_level() -> usize { todo!() }

    pub fn get(&self, key: &K) -> Option<&V> { todo!() }

    pub fn insert(&mut self, key: K, value: V) { todo!() }

    pub fn remove(&mut self, key: &K) -> bool { todo!() }

    /// Range scan: returns all (key, value) pairs where from <= key <= to.
    pub fn range(&self, from: &K, to: &K) -> Vec<(K, V)> { todo!() }
}

impl<K: Ord, V> Drop for SkipList<K, V> {
    fn drop(&mut self) { todo!() }
}

impl<K: Ord + Clone + Default, V: Clone + Default> Default for SkipList<K, V> {
    fn default() -> Self { Self::new() }
}

/// Xorshift64 PRNG coin flip. Use this in random_level().
fn rand_bool() -> bool {
    use std::cell::Cell;
    thread_local!(static STATE: Cell<u64> = Cell::new(0x517cc1b727220a95));
    STATE.with(|s| {
        let mut x = s.get();
        x ^= x >> 12; x ^= x << 25; x ^= x >> 27;
        s.set(x);
        x & 1 == 0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod basic {
        use super::*;

        #[test]
        fn get_on_empty_returns_none() {
            let s: SkipList<i32, i32> = SkipList::new();
            assert_eq!(s.get(&1), None);
        }

        #[test]
        fn insert_then_get() {
            let mut s = SkipList::new();
            s.insert(42, "hello");
            assert_eq!(s.get(&42), Some(&"hello"));
        }

        #[test]
        fn update_existing_key() {
            let mut s = SkipList::new();
            s.insert(1, "old");
            s.insert(1, "new");
            assert_eq!(s.get(&1), Some(&"new"));
            assert_eq!(s.len(), 1);
        }

        #[test]
        fn missing_key_returns_none() {
            let mut s = SkipList::new();
            s.insert(1, "a");
            assert_eq!(s.get(&2), None);
        }
    }

    mod ordering {
        use super::*;

        #[test]
        fn many_insertions_all_retrievable() {
            let mut s = SkipList::new();
            for i in 0..200i32 {
                s.insert(i, i * 2);
            }
            for i in 0..200i32 {
                assert_eq!(s.get(&i), Some(&(i * 2)), "missing {i}");
            }
        }

        #[test]
        fn reverse_insertion_all_retrievable() {
            let mut s = SkipList::new();
            for i in (0..100i32).rev() {
                s.insert(i, i);
            }
            for i in 0..100i32 {
                assert_eq!(s.get(&i), Some(&i));
            }
        }
    }

    mod remove {
        use super::*;

        #[test]
        fn remove_existing_key_returns_true() {
            let mut s = SkipList::new();
            s.insert(1, "a");
            assert!(s.remove(&1));
        }

        #[test]
        fn remove_missing_key_returns_false() {
            let mut s: SkipList<i32, i32> = SkipList::new();
            assert!(!s.remove(&99));
        }

        #[test]
        fn removed_key_is_gone() {
            let mut s = SkipList::new();
            s.insert(1, "a");
            s.remove(&1);
            assert_eq!(s.get(&1), None);
        }

        #[test]
        fn remove_decrements_len() {
            let mut s = SkipList::new();
            s.insert(1, "a");
            s.remove(&1);
            assert_eq!(s.len(), 0);
        }
    }

    mod range {
        use super::*;

        #[test]
        fn range_returns_sorted_subset() {
            let mut s = SkipList::new();
            for i in 1..=10i32 { s.insert(i, i * 10); }
            let result = s.range(&3, &6);
            let keys: Vec<i32> = result.iter().map(|(k, _)| *k).collect();
            assert_eq!(keys, vec![3, 4, 5, 6]);
        }

        #[test]
        fn range_with_no_matches_returns_empty() {
            let mut s: SkipList<i32, ()> = SkipList::new();
            for i in 1..=5 { s.insert(i, ()); }
            assert!(s.range(&10, &20).is_empty());
        }
    }
}

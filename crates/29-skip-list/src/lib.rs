const MAX_LEVEL: usize = 16;

struct Node<K, V> {
    key: K,
    value: V,
    /// next[i] is a raw pointer to the next node at level i.
    /// Ownership of all nodes flows through the level-0 chain only.
    next: Vec<*mut Node<K, V>>,
}

pub struct SkipList<K: Ord, V> {
    /// Sentinel head; its key/value are never accessed.
    head: Box<Node<K, V>>,
    level: usize,
    len: usize,
}

// SAFETY: SkipList owns all nodes exclusively via the level-0 chain.
// Raw pointers at higher levels never outlive the node they point to.
unsafe impl<K: Ord + Send, V: Send> Send for SkipList<K, V> {}

/// Safely produce a `&Node` from a non-null raw pointer.
/// The caller must ensure the pointer is valid and no mutable alias exists.
#[inline(always)]
unsafe fn node_ref<'a, K, V>(ptr: *mut Node<K, V>) -> &'a Node<K, V> {
    // SAFETY: caller guarantees ptr is non-null and valid for shared access.
    // Using &*ptr is explicit, not implicit autoref.
    &*ptr
}

impl<K: Ord + Clone + Default, V: Clone + Default> SkipList<K, V> {
    pub fn new() -> Self {
        let head = Box::new(Node {
            key: K::default(),
            value: V::default(),
            next: vec![std::ptr::null_mut(); MAX_LEVEL],
        });
        Self { head, level: 1, len: 0 }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    fn random_level() -> usize {
        let mut lvl = 1;
        while lvl < MAX_LEVEL && rand_bool() { lvl += 1; }
        lvl
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        // SAFETY: head is always valid; next pointers are valid while SkipList is alive.
        unsafe {
            let mut curr: *const Node<K, V> = &*self.head;
            for i in (0..self.level).rev() {
                loop {
                    let next_ptr = node_ref(curr as *mut Node<K, V>).next[i];
                    if next_ptr.is_null() { break; }
                    let next = node_ref(next_ptr);
                    if &next.key < key {
                        curr = next_ptr;
                    } else {
                        break;
                    }
                }
            }
            let next_ptr = node_ref(curr as *mut Node<K, V>).next[0];
            if !next_ptr.is_null() {
                let next = node_ref(next_ptr);
                if &next.key == key {
                    return Some(&next.value);
                }
            }
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut update: Vec<*mut Node<K, V>> = vec![std::ptr::null_mut(); MAX_LEVEL];
        // SAFETY: head is always valid; we hold &mut self so no other references exist.
        unsafe {
            let mut curr: *mut Node<K, V> = &mut *self.head;
            for i in (0..self.level).rev() {
                loop {
                    let next_ptr = node_ref(curr).next[i];
                    if next_ptr.is_null() { break; }
                    let next = node_ref(next_ptr);
                    if next.key < key {
                        curr = next_ptr;
                    } else {
                        break;
                    }
                }
                update[i] = curr;
            }

            // Check for existing key
            let next_ptr = node_ref(curr).next[0];
            if !next_ptr.is_null() {
                let next = &mut *next_ptr;
                if next.key == key {
                    next.value = value;
                    return;
                }
            }

            let new_level = Self::random_level();
            if new_level > self.level {
                for i in self.level..new_level {
                    update[i] = &mut *self.head;
                }
                self.level = new_level;
            }

            let new_node = Box::into_raw(Box::new(Node {
                key,
                value,
                next: vec![std::ptr::null_mut(); new_level],
            }));

            for i in 0..new_level {
                let new = &mut *new_node;
                let pred = &mut *update[i];
                new.next[i] = pred.next[i];
                pred.next[i] = new_node;
            }
        }
        self.len += 1;
    }

    pub fn remove(&mut self, key: &K) -> bool {
        let mut update: Vec<*mut Node<K, V>> = vec![std::ptr::null_mut(); MAX_LEVEL];
        // SAFETY: head is always valid; we hold &mut self.
        unsafe {
            let mut curr: *mut Node<K, V> = &mut *self.head;
            for i in (0..self.level).rev() {
                loop {
                    let next_ptr = node_ref(curr).next[i];
                    if next_ptr.is_null() { break; }
                    let next = node_ref(next_ptr);
                    if &next.key < key {
                        curr = next_ptr;
                    } else {
                        break;
                    }
                }
                update[i] = curr;
            }

            let target = node_ref(curr).next[0];
            if target.is_null() || &node_ref(target).key != key {
                return false;
            }

            for i in 0..self.level {
                let pred = &mut *update[i];
                if pred.next[i] != target { break; }
                pred.next[i] = node_ref(target).next[i];
            }

            // Reclaim node -- ownership flows through the level-0 chain
            drop(Box::from_raw(target));
        }
        self.len -= 1;

        // Shrink level if top levels are empty
        while self.level > 1 && self.head.next[self.level - 1].is_null() {
            self.level -= 1;
        }
        true
    }

    /// Range scan: returns all (key, value) pairs where from <= key <= to.
    pub fn range(&self, from: &K, to: &K) -> Vec<(K, V)> {
        // SAFETY: all pointers are valid while SkipList is alive.
        unsafe {
            let mut curr: *const Node<K, V> = &*self.head;
            for i in (0..self.level).rev() {
                loop {
                    let next_ptr = node_ref(curr as *mut Node<K, V>).next[i];
                    if next_ptr.is_null() { break; }
                    let next = node_ref(next_ptr);
                    if &next.key < from {
                        curr = next_ptr;
                    } else {
                        break;
                    }
                }
            }
            let mut out = Vec::new();
            let mut ptr = node_ref(curr as *mut Node<K, V>).next[0];
            while !ptr.is_null() {
                let node = node_ref(ptr);
                if &node.key > to { break; }
                out.push((node.key.clone(), node.value.clone()));
                ptr = node.next[0];
            }
            out
        }
    }
}

impl<K: Ord, V> Drop for SkipList<K, V> {
    fn drop(&mut self) {
        // SAFETY: we walk the level-0 chain and reclaim each node exactly once.
        unsafe {
            let mut curr = self.head.next[0];
            while !curr.is_null() {
                let next = node_ref(curr).next[0];
                drop(Box::from_raw(curr));
                curr = next;
            }
            // Null out head's pointers so Box<Node> drop doesn't attempt to free them
            for p in &mut self.head.next {
                *p = std::ptr::null_mut();
            }
        }
    }
}

impl<K: Ord + Clone + Default, V: Clone + Default> Default for SkipList<K, V> {
    fn default() -> Self { Self::new() }
}

/// Xorshift64 PRNG coin flip -- avoids thread_rng dependency; deterministic under test.
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

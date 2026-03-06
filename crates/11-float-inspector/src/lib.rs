use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_CAP: usize = 16;
const MAX_LOAD: f64 = 0.70;

#[derive(Clone)]
enum Slot<K, V> {
    Empty,
    Occupied { key: K, value: V, dist: usize }, // dist = probe distance from ideal slot
}

pub struct HashMap<K, V> {
    slots: Vec<Slot<K, V>>,
    len: usize,
}

impl<K: Eq + Hash + Clone, V: Clone> HashMap<K, V> {
    pub fn new() -> Self {
        Self { slots: vec![Slot::Empty; INITIAL_CAP], len: 0 }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn load_factor(&self) -> f64 {
        self.len as f64 / self.slots.len() as f64
    }

    fn hash_index(&self, key: &K) -> usize {
        let mut h = DefaultHasher::new();
        key.hash(&mut h);
        h.finish() as usize % self.slots.len()
    }

    /// Insert or update. Returns the old value if the key already existed.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.load_factor() >= MAX_LOAD {
            self.resize();
        }
        self.insert_inner(key, value)
    }

    fn insert_inner(&mut self, mut key: K, mut value: V) -> Option<V> {
        let cap = self.slots.len();
        let mut idx = self.hash_index(&key);
        let mut dist = 0usize;

        loop {
            // Take ownership of the current slot to avoid borrow conflicts
            let current = std::mem::replace(&mut self.slots[idx], Slot::Empty);

            // Destructure in a guard-free match, then decide what to do with the data
            let (k, v, d) = match current {
                Slot::Empty => {
                    self.slots[idx] = Slot::Occupied { key, value, dist };
                    self.len += 1;
                    return None;
                }
                Slot::Occupied { key: k, value: v, dist: d } => (k, v, d),
            };

            if k == key {
                // Update existing key
                self.slots[idx] = Slot::Occupied { key: k, value, dist: d };
                return Some(v);
            } else if dist > d {
                // Robin Hood: place our entry here, continue inserting the displaced entry
                self.slots[idx] = Slot::Occupied { key, value, dist };
                key = k;
                value = v;
                dist = d + 1;
            } else {
                // Keep the existing entry and advance
                self.slots[idx] = Slot::Occupied { key: k, value: v, dist: d };
                dist += 1;
            }

            idx = (idx + 1) % cap;
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let cap = self.slots.len();
        let mut idx = self.hash_index(key);
        let mut dist = 0;
        loop {
            match &self.slots[idx] {
                Slot::Empty => return None,
                Slot::Occupied { dist: occ_dist, .. } if dist > *occ_dist => {
                    // Robin Hood invariant: if we've probed further than the occupant,
                    // the key cannot be here
                    return None;
                }
                Slot::Occupied { key: k, value: v, .. } if k == key => return Some(v),
                _ => {}
            }
            dist += 1;
            idx = (idx + 1) % cap;
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Remove a key, using backward-shift deletion to maintain Robin Hood invariant.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let cap = self.slots.len();
        let mut idx = self.hash_index(key);
        let mut dist = 0;

        // Find the slot
        let found_idx = loop {
            match &self.slots[idx] {
                Slot::Empty => return None,
                Slot::Occupied { dist: occ_dist, .. } if dist > *occ_dist => return None,
                Slot::Occupied { key: k, .. } if k == key => break idx,
                _ => {}
            }
            dist += 1;
            idx = (idx + 1) % cap;
        };

        let removed = if let Slot::Occupied { value, .. } =
            std::mem::replace(&mut self.slots[found_idx], Slot::Empty)
        {
            value
        } else {
            unreachable!()
        };
        self.len -= 1;

        // Backward shift: pull subsequent entries one position back
        let mut current = found_idx;
        loop {
            let next = (current + 1) % cap;
            match &self.slots[next] {
                Slot::Empty => break,
                Slot::Occupied { dist: 0, .. } => break, // at ideal slot, can't move back
                _ => {}
            }
            // Move next into current, update its dist
            let mut entry = std::mem::replace(&mut self.slots[next], Slot::Empty);
            if let Slot::Occupied { dist, .. } = &mut entry {
                *dist -= 1;
            }
            self.slots[current] = entry;
            current = next;
        }

        Some(removed)
    }

    fn resize(&mut self) {
        let new_cap = self.slots.len() * 2;
        let old_slots = std::mem::replace(&mut self.slots, vec![Slot::Empty; new_cap]);
        self.len = 0;
        for slot in old_slots {
            if let Slot::Occupied { key, value, .. } = slot {
                self.insert_inner(key, value);
            }
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for HashMap<K, V> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod basic {
        use super::*;

        #[test]
        fn insert_and_get() {
            let mut m: HashMap<&str, i32> = HashMap::new();
            m.insert("hello", 42);
            assert_eq!(m.get(&"hello"), Some(&42));
        }

        #[test]
        fn get_missing_key_returns_none() {
            let m: HashMap<&str, i32> = HashMap::new();
            assert_eq!(m.get(&"missing"), None);
        }

        #[test]
        fn update_existing_key_returns_old_value() {
            let mut m: HashMap<&str, i32> = HashMap::new();
            m.insert("k", 1);
            let old = m.insert("k", 2);
            assert_eq!(old, Some(1));
            assert_eq!(m.get(&"k"), Some(&2));
        }

        #[test]
        fn len_tracks_insertions() {
            let mut m: HashMap<i32, i32> = HashMap::new();
            m.insert(1, 10);
            m.insert(2, 20);
            assert_eq!(m.len(), 2);
        }

        #[test]
        fn update_does_not_increase_len() {
            let mut m: HashMap<i32, i32> = HashMap::new();
            m.insert(1, 10);
            m.insert(1, 20);
            assert_eq!(m.len(), 1);
        }
    }

    mod remove {
        use super::*;

        #[test]
        fn remove_existing_key_returns_value() {
            let mut m: HashMap<&str, i32> = HashMap::new();
            m.insert("x", 99);
            assert_eq!(m.remove(&"x"), Some(99));
        }

        #[test]
        fn remove_missing_key_returns_none() {
            let mut m: HashMap<&str, i32> = HashMap::new();
            assert_eq!(m.remove(&"x"), None);
        }

        #[test]
        fn removed_key_is_no_longer_found() {
            let mut m: HashMap<&str, i32> = HashMap::new();
            m.insert("x", 1);
            m.remove(&"x");
            assert_eq!(m.get(&"x"), None);
        }

        #[test]
        fn remove_decrements_len() {
            let mut m: HashMap<i32, i32> = HashMap::new();
            m.insert(1, 1);
            m.remove(&1);
            assert_eq!(m.len(), 0);
        }
    }

    mod resize {
        use super::*;

        #[test]
        fn all_entries_survive_resize() {
            let mut m: HashMap<i32, i32> = HashMap::new();
            for i in 0..50 {
                m.insert(i, i * 10);
            }
            for i in 0..50 {
                assert_eq!(m.get(&i), Some(&(i * 10)), "missing key {i}");
            }
        }

        #[test]
        fn load_factor_stays_below_max_after_resize() {
            let mut m: HashMap<i32, i32> = HashMap::new();
            for i in 0..100 {
                m.insert(i, i);
            }
            assert!(m.load_factor() < 0.75);
        }
    }

    mod collisions {
        use super::*;

        #[test]
        fn many_colliding_keys_all_retrievable() {
            // Insert many numeric keys which will have different hash slots
            let mut m: HashMap<u64, u64> = HashMap::new();
            for i in 0..200u64 {
                m.insert(i, i * 2);
            }
            for i in 0..200u64 {
                assert_eq!(m.get(&i), Some(&(i * 2)));
            }
        }
    }
}

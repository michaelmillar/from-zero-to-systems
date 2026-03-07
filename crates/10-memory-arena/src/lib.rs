// ============================================================
//  YOUR CHALLENGE - implement a bump allocator (arena).
//  A bump allocator works by keeping a pointer into a block
//  of memory and "bumping" it forward on each allocation.
//  Deallocating individual objects is impossible - the whole
//  arena is freed at once. This makes it extremely fast.
//
//  Rules:
//  - Arena::new(capacity) allocates a Vec<u8> of that size
//  - Arena::alloc::<T>() returns Option<&mut T>
//      - Returns None if there isn't enough space remaining
//      - The returned reference must not outlive the arena
//  - Arena::reset() resets the bump pointer to 0 (reuses memory)
//  - Arena::used() returns how many bytes have been allocated
//  - Arena::remaining() returns how many bytes are still free
//
//  You will need `unsafe` to cast raw pointers.
//  Hint: align the offset before allocating:
//    let aligned = (self.offset + align - 1) & !(align - 1);
// ============================================================

pub struct Arena {
    buffer: Vec<u8>,
    offset: usize,
}

impl Arena {
    /// Create a new arena with `capacity` bytes of backing storage.
    pub fn new(capacity: usize) -> Self {
        Self { buffer: vec![0u8; capacity], offset: 0 }
    }

    /// Allocate space for one value of type T, aligned to T's alignment.
    /// Returns a mutable reference to uninitialised memory, or None if
    /// the arena is full.
    pub fn alloc<T>(&mut self) -> Option<&mut T> {
        todo!()
    }

    /// Reset the bump pointer - logically frees all allocations.
    /// The backing memory is reused on the next alloc.
    pub fn reset(&mut self) {
        todo!()
    }

    /// Number of bytes currently allocated.
    pub fn used(&self) -> usize {
        todo!()
    }

    /// Number of bytes still available.
    pub fn remaining(&self) -> usize {
        todo!()
    }
}

// ============================================================
//  TESTS - these ARE the specification.
//  Run `cargo test -p memory-arena` to see them fail.
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod capacity_and_bookkeeping {
        use super::*;

        #[test]
        fn given_fresh_arena_used_is_zero() {
            let arena = Arena::new(256);
            assert_eq!(arena.used(), 0);
        }

        #[test]
        fn given_fresh_arena_remaining_equals_capacity() {
            let arena = Arena::new(256);
            assert_eq!(arena.remaining(), 256);
        }

        #[test]
        fn after_reset_used_returns_to_zero() {
            let mut arena = Arena::new(256);
            let _ = arena.alloc::<u64>();
            arena.reset();
            assert_eq!(arena.used(), 0);
        }
    }

    mod allocating_primitives {
        use super::*;

        #[test]
        fn allocating_u64_reduces_remaining_by_at_least_8_bytes() {
            let mut arena = Arena::new(256);
            let before = arena.remaining();
            let _ = arena.alloc::<u64>();
            assert!(arena.remaining() <= before - 8);
        }

        #[test]
        fn allocated_value_can_be_written_and_read_back() {
            let mut arena = Arena::new(256);
            let slot = arena.alloc::<u32>().expect("should have space");
            *slot = 0xDEAD_BEEF;
            assert_eq!(*slot, 0xDEAD_BEEF);
        }

        #[test]
        fn multiple_allocations_do_not_overlap() {
            let mut arena = Arena::new(256);
            let a = arena.alloc::<u32>().unwrap() as *mut u32;
            let b = arena.alloc::<u32>().unwrap() as *mut u32;
            assert_ne!(a, b);
        }
    }

    mod exhaustion {
        use super::*;

        #[test]
        fn alloc_returns_none_when_arena_is_full() {
            let mut arena = Arena::new(4); // only 4 bytes
            // First u32 should succeed
            assert!(arena.alloc::<u32>().is_some());
            // Second u32 should fail
            assert!(arena.alloc::<u32>().is_none());
        }

        #[test]
        fn after_reset_allocation_succeeds_again() {
            let mut arena = Arena::new(4);
            let _ = arena.alloc::<u32>();
            arena.reset();
            assert!(arena.alloc::<u32>().is_some());
        }
    }

    mod structs {
        use super::*;

        #[derive(Debug, PartialEq)]
        struct Order { price: f64, qty: u32, side: u8 }

        #[test]
        fn can_allocate_and_store_a_struct() {
            let mut arena = Arena::new(1024);
            let order = arena.alloc::<Order>().unwrap();
            *order = Order { price: 100.50, qty: 500, side: 1 };
            assert_eq!(order.price, 100.50);
            assert_eq!(order.qty, 500);
        }
    }
}

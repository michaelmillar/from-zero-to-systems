// ============================================================
//  YOUR CHALLENGE - implement a buffer pool with clock eviction.
//
//  frames: fixed Vec<Frame> (capacity set at construction).
//  page_table: HashMap<PageId, frame_index> -- fast lookup.
//  clock_hand: sweeps frames; if ref_bit set, clear it (second chance);
//              if pin_count > 0, skip; otherwise evict.
//  pin: load from disk if not cached, increment pin_count, set ref_bit.
//  unpin: decrement pin_count, set dirty flag if is_dirty.
//  flush_dirty: write dirty unpinned pages to disk, clear dirty flag.
// ============================================================

use std::collections::HashMap;
use slotted_page::PAGE_SIZE;

pub type PageId = usize;

#[derive(Clone)]
struct Frame {
    page_id: Option<PageId>,
    data: Vec<u8>,
    dirty: bool,
    ref_bit: bool,
    pin_count: usize,
}

impl Frame {
    fn empty() -> Self {
        Self { page_id: None, data: vec![0u8; PAGE_SIZE], dirty: false, ref_bit: false, pin_count: 0 }
    }
}

/// Simulated disk: a growable list of PAGE_SIZE byte pages.
pub struct Disk {
    pages: Vec<Vec<u8>>,
}

impl Disk {
    pub fn new() -> Self { Self { pages: Vec::new() } }

    pub fn alloc_page(&mut self) -> PageId {
        let id = self.pages.len();
        self.pages.push(vec![0u8; PAGE_SIZE]);
        id
    }

    pub fn read(&self, page_id: PageId) -> &[u8] { &self.pages[page_id] }

    pub fn write(&mut self, page_id: PageId, data: &[u8]) {
        self.pages[page_id].copy_from_slice(data);
    }
}

impl Default for Disk {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, PartialEq)]
pub enum PoolError {
    AllFramesPinned,
    PageNotPinned,
}

pub struct BufferPool {
    frames: Vec<Frame>,
    page_table: HashMap<PageId, usize>,
    clock_hand: usize,
    capacity: usize,
}

impl BufferPool {
    pub fn new(capacity: usize) -> Self { todo!() }

    /// Pin a page into a frame. Returns a mutable reference to the frame's data.
    /// Fetches from disk if not already cached.
    pub fn pin(&mut self, page_id: PageId, disk: &mut Disk) -> Result<&mut Vec<u8>, PoolError> { todo!() }

    /// Unpin a page, marking it dirty if modified.
    pub fn unpin(&mut self, page_id: PageId, is_dirty: bool) -> Result<(), PoolError> { todo!() }

    /// Write all dirty, unpinned pages back to disk.
    pub fn flush_dirty(&mut self, disk: &mut Disk) { todo!() }

    pub fn is_cached(&self, page_id: PageId) -> bool { todo!() }

    pub fn dirty_count(&self) -> usize { todo!() }

    fn evict(&mut self, disk: &mut Disk) -> Result<usize, PoolError> { todo!() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(pool_size: usize, n_pages: usize) -> (BufferPool, Disk) {
        let mut disk = Disk::new();
        for _ in 0..n_pages { disk.alloc_page(); }
        (BufferPool::new(pool_size), disk)
    }

    mod pin_unpin {
        use super::*;

        #[test]
        fn pin_loads_page_into_pool() {
            let (mut pool, mut disk) = setup(4, 4);
            pool.pin(0, &mut disk).unwrap();
            assert!(pool.is_cached(0));
        }

        #[test]
        fn data_written_to_pinned_page_is_visible() {
            let (mut pool, mut disk) = setup(4, 4);
            let data = pool.pin(0, &mut disk).unwrap();
            data[0] = 42;
            assert_eq!(pool.pin(0, &mut disk).unwrap()[0], 42);
        }

        #[test]
        fn unpin_marks_page_dirty() {
            let (mut pool, mut disk) = setup(4, 4);
            pool.pin(0, &mut disk).unwrap();
            pool.unpin(0, true).unwrap();
            assert_eq!(pool.dirty_count(), 1);
        }

        #[test]
        fn unpin_non_cached_page_returns_error() {
            let (mut pool, _) = setup(4, 4);
            assert_eq!(pool.unpin(0, false), Err(PoolError::PageNotPinned));
        }
    }

    mod eviction {
        use super::*;

        #[test]
        fn evicts_when_pool_is_full() {
            let (mut pool, mut disk) = setup(2, 4);
            pool.pin(0, &mut disk).unwrap();
            pool.unpin(0, false).unwrap();
            pool.pin(1, &mut disk).unwrap();
            pool.unpin(1, false).unwrap();
            assert!(pool.pin(2, &mut disk).is_ok());
        }

        #[test]
        fn pinned_pages_are_not_evicted() {
            let (mut pool, mut disk) = setup(2, 4);
            pool.pin(0, &mut disk).unwrap();
            pool.pin(1, &mut disk).unwrap();
            pool.unpin(1, false).unwrap();
            pool.pin(2, &mut disk).unwrap();
            assert!(pool.is_cached(0));
        }

        #[test]
        fn all_pinned_returns_error() {
            let (mut pool, mut disk) = setup(2, 4);
            pool.pin(0, &mut disk).unwrap();
            pool.pin(1, &mut disk).unwrap();
            assert_eq!(pool.pin(2, &mut disk), Err(PoolError::AllFramesPinned));
        }
    }

    mod flush {
        use super::*;

        #[test]
        fn flush_writes_dirty_pages_to_disk() {
            let (mut pool, mut disk) = setup(4, 4);
            let data = pool.pin(0, &mut disk).unwrap();
            data[0] = 99;
            pool.unpin(0, true).unwrap();
            pool.flush_dirty(&mut disk);
            assert_eq!(disk.read(0)[0], 99);
        }

        #[test]
        fn dirty_count_is_zero_after_flush() {
            let (mut pool, mut disk) = setup(4, 4);
            pool.pin(0, &mut disk).unwrap();
            pool.unpin(0, true).unwrap();
            pool.flush_dirty(&mut disk);
            assert_eq!(pool.dirty_count(), 0);
        }
    }
}

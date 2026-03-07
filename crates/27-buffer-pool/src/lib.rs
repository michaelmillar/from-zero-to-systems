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

    pub fn read(&self, page_id: PageId) -> &[u8] {
        &self.pages[page_id]
    }

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
    page_table: HashMap<PageId, usize>, // page_id -> frame_index
    clock_hand: usize,
    capacity: usize,
}

impl BufferPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            frames: (0..capacity).map(|_| Frame::empty()).collect(),
            page_table: HashMap::new(),
            clock_hand: 0,
            capacity,
        }
    }

    /// Pin a page into a frame. Returns a mutable reference to the frame's data.
    /// Fetches from disk if not already cached.
    pub fn pin(&mut self, page_id: PageId, disk: &mut Disk) -> Result<&mut Vec<u8>, PoolError> {
        if let Some(&frame_idx) = self.page_table.get(&page_id) {
            self.frames[frame_idx].pin_count += 1;
            self.frames[frame_idx].ref_bit = true;
            return Ok(&mut self.frames[frame_idx].data);
        }
        // Find a victim frame via clock algorithm
        let victim = self.evict(disk)?;
        // Load from disk
        let page_data = disk.read(page_id).to_vec();
        self.frames[victim].data.copy_from_slice(&page_data);
        self.frames[victim].page_id = Some(page_id);
        self.frames[victim].dirty = false;
        self.frames[victim].pin_count = 1;
        self.frames[victim].ref_bit = true;
        self.page_table.insert(page_id, victim);
        Ok(&mut self.frames[victim].data)
    }

    /// Unpin a page, marking it dirty if modified.
    pub fn unpin(&mut self, page_id: PageId, is_dirty: bool) -> Result<(), PoolError> {
        let &frame_idx = self.page_table.get(&page_id).ok_or(PoolError::PageNotPinned)?;
        if self.frames[frame_idx].pin_count == 0 {
            return Err(PoolError::PageNotPinned);
        }
        self.frames[frame_idx].pin_count -= 1;
        if is_dirty {
            self.frames[frame_idx].dirty = true;
        }
        Ok(())
    }

    /// Write all dirty, unpinned pages back to disk.
    pub fn flush_dirty(&mut self, disk: &mut Disk) {
        for frame in &mut self.frames {
            if frame.dirty && frame.pin_count == 0 {
                if let Some(pid) = frame.page_id {
                    disk.write(pid, &frame.data);
                    frame.dirty = false;
                }
            }
        }
    }

    pub fn is_cached(&self, page_id: PageId) -> bool {
        self.page_table.contains_key(&page_id)
    }

    pub fn dirty_count(&self) -> usize {
        self.frames.iter().filter(|f| f.dirty).count()
    }

    fn evict(&mut self, disk: &mut Disk) -> Result<usize, PoolError> {
        let cap = self.capacity;
        for _ in 0..cap * 2 {
            let idx = self.clock_hand % cap;
            self.clock_hand = (self.clock_hand + 1) % cap;
            let frame = &mut self.frames[idx];
            if frame.pin_count > 0 {
                continue;
            }
            if frame.ref_bit {
                frame.ref_bit = false; // second chance
                continue;
            }
            // Evict this frame
            if frame.dirty {
                if let Some(pid) = frame.page_id {
                    disk.write(pid, &frame.data);
                }
                frame.dirty = false;
            }
            if let Some(pid) = frame.page_id.take() {
                self.page_table.remove(&pid);
            }
            return Ok(idx);
        }
        Err(PoolError::AllFramesPinned)
    }
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
            // Pool is full but unpinned; pinning page 2 should evict one
            assert!(pool.pin(2, &mut disk).is_ok());
        }

        #[test]
        fn pinned_pages_are_not_evicted() {
            let (mut pool, mut disk) = setup(2, 4);
            pool.pin(0, &mut disk).unwrap(); // pinned, never evicted
            pool.pin(1, &mut disk).unwrap();
            pool.unpin(1, false).unwrap();
            // pin page 2: should evict page 1 (unpinned), not page 0 (pinned)
            pool.pin(2, &mut disk).unwrap();
            assert!(pool.is_cached(0)); // page 0 still in pool
        }

        #[test]
        fn all_pinned_returns_error() {
            let (mut pool, mut disk) = setup(2, 4);
            pool.pin(0, &mut disk).unwrap(); // pinned
            pool.pin(1, &mut disk).unwrap(); // pinned
            // Both pinned, no eviction possible
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

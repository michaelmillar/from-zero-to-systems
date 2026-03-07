pub const PAGE_SIZE: usize = 4096;
const HEADER: usize = 4;   // num_slots (u16) + free_ptr (u16)
const SLOT_SZ: usize = 4;  // offset (u16) + length (u16)

pub type SlotId = usize;

#[derive(Debug, PartialEq)]
pub enum PageError {
    OutOfSpace,
    InvalidSlot,
}

pub struct SlottedPage {
    data: Vec<u8>,
}

impl SlottedPage {
    pub fn new() -> Self {
        let mut data = vec![0u8; PAGE_SIZE];
        // free_ptr = PAGE_SIZE (all data space available)
        let fp = PAGE_SIZE as u16;
        data[2..4].copy_from_slice(&fp.to_le_bytes());
        Self { data }
    }

    fn num_slots(&self) -> usize {
        u16::from_le_bytes([self.data[0], self.data[1]]) as usize
    }

    fn free_ptr(&self) -> usize {
        u16::from_le_bytes([self.data[2], self.data[3]]) as usize
    }

    fn set_num_slots(&mut self, n: usize) {
        self.data[0..2].copy_from_slice(&(n as u16).to_le_bytes());
    }

    fn set_free_ptr(&mut self, p: usize) {
        self.data[2..4].copy_from_slice(&(p as u16).to_le_bytes());
    }

    fn read_slot(&self, id: SlotId) -> (usize, usize) {
        let base = HEADER + id * SLOT_SZ;
        let off = u16::from_le_bytes([self.data[base], self.data[base + 1]]) as usize;
        let len = u16::from_le_bytes([self.data[base + 2], self.data[base + 3]]) as usize;
        (off, len)
    }

    fn write_slot(&mut self, id: SlotId, off: usize, len: usize) {
        let base = HEADER + id * SLOT_SZ;
        self.data[base..base + 2].copy_from_slice(&(off as u16).to_le_bytes());
        self.data[base + 2..base + 4].copy_from_slice(&(len as u16).to_le_bytes());
    }

    /// Bytes available for new record data (excluding a potential new slot entry).
    pub fn free_space(&self) -> usize {
        let slot_end = HEADER + self.num_slots() * SLOT_SZ;
        self.free_ptr().saturating_sub(slot_end)
    }

    /// Insert `record` and return its SlotId.
    /// Reuses a deleted slot if one exists; otherwise appends a new slot entry.
    pub fn insert(&mut self, record: &[u8]) -> Result<SlotId, PageError> {
        let n = self.num_slots();
        let reuse = (0..n).find(|&s| self.read_slot(s).1 == 0);
        let extra_slot = if reuse.is_some() { 0 } else { SLOT_SZ };
        let needed = record.len() + extra_slot;
        let slot_end = HEADER + n * SLOT_SZ;
        if self.free_ptr() < slot_end + needed {
            return Err(PageError::OutOfSpace);
        }
        let new_fp = self.free_ptr() - record.len();
        self.data[new_fp..new_fp + record.len()].copy_from_slice(record);
        self.set_free_ptr(new_fp);
        let id = reuse.unwrap_or_else(|| { self.set_num_slots(n + 1); n });
        self.write_slot(id, new_fp, record.len());
        Ok(id)
    }

    /// Read a record by SlotId.
    pub fn get(&self, id: SlotId) -> Result<&[u8], PageError> {
        if id >= self.num_slots() {
            return Err(PageError::InvalidSlot);
        }
        let (off, len) = self.read_slot(id);
        if len == 0 {
            return Err(PageError::InvalidSlot);
        }
        Ok(&self.data[off..off + len])
    }

    /// Mark a slot as deleted (length = 0). Space is not reclaimed until compact().
    pub fn delete(&mut self, id: SlotId) -> Result<(), PageError> {
        if id >= self.num_slots() || self.read_slot(id).1 == 0 {
            return Err(PageError::InvalidSlot);
        }
        self.write_slot(id, 0, 0);
        Ok(())
    }

    /// Defragment: rewrite all live records contiguously, reclaiming deleted space.
    /// Slot IDs of live records are preserved.
    pub fn compact(&mut self) {
        let n = self.num_slots();
        let live: Vec<(SlotId, Vec<u8>)> = (0..n)
            .filter_map(|s| {
                let (off, len) = self.read_slot(s);
                if len > 0 { Some((s, self.data[off..off + len].to_vec())) } else { None }
            })
            .collect();
        self.set_free_ptr(PAGE_SIZE);
        for (id, rec) in live {
            let new_fp = self.free_ptr() - rec.len();
            self.data[new_fp..new_fp + rec.len()].copy_from_slice(&rec);
            self.set_free_ptr(new_fp);
            self.write_slot(id, new_fp, rec.len());
        }
    }
}

impl Default for SlottedPage {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod insert_get {
        use super::*;

        #[test]
        fn insert_and_retrieve_single_record() {
            let mut p = SlottedPage::new();
            let id = p.insert(b"hello").unwrap();
            assert_eq!(p.get(id).unwrap(), b"hello");
        }

        #[test]
        fn multiple_records_are_independent() {
            let mut p = SlottedPage::new();
            let a = p.insert(b"alpha").unwrap();
            let b = p.insert(b"beta").unwrap();
            assert_eq!(p.get(a).unwrap(), b"alpha");
            assert_eq!(p.get(b).unwrap(), b"beta");
        }

        #[test]
        fn slot_ids_are_sequential() {
            let mut p = SlottedPage::new();
            assert_eq!(p.insert(b"a").unwrap(), 0);
            assert_eq!(p.insert(b"b").unwrap(), 1);
            assert_eq!(p.insert(b"c").unwrap(), 2);
        }

        #[test]
        fn out_of_space_returns_error() {
            let mut p = SlottedPage::new();
            let big = vec![0u8; 4000];
            p.insert(&big).unwrap();
            assert_eq!(p.insert(&big), Err(PageError::OutOfSpace));
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn deleted_slot_returns_invalid_slot_error() {
            let mut p = SlottedPage::new();
            let id = p.insert(b"data").unwrap();
            p.delete(id).unwrap();
            assert_eq!(p.get(id), Err(PageError::InvalidSlot));
        }

        #[test]
        fn delete_invalid_slot_returns_error() {
            let mut p = SlottedPage::new();
            assert_eq!(p.delete(99), Err(PageError::InvalidSlot));
        }

        #[test]
        fn deleted_slot_id_is_reused_on_next_insert() {
            let mut p = SlottedPage::new();
            let id0 = p.insert(b"first").unwrap();
            p.delete(id0).unwrap();
            let id1 = p.insert(b"second").unwrap();
            assert_eq!(id0, id1); // same slot reused
        }
    }

    mod compact {
        use super::*;

        #[test]
        fn live_records_survive_compaction() {
            let mut p = SlottedPage::new();
            let a = p.insert(b"keep me").unwrap();
            let b = p.insert(b"delete me").unwrap();
            let c = p.insert(b"keep me too").unwrap();
            p.delete(b).unwrap();
            p.compact();
            assert_eq!(p.get(a).unwrap(), b"keep me");
            assert_eq!(p.get(c).unwrap(), b"keep me too");
        }

        #[test]
        fn compact_reclaims_space_for_new_inserts() {
            let mut p = SlottedPage::new();
            let big = vec![42u8; 1500];
            let id = p.insert(&big).unwrap();
            p.insert(&big).unwrap();
            p.delete(id).unwrap();
            // Without compact, third insert might fail; after compact it must succeed
            p.compact();
            assert!(p.insert(&big).is_ok());
        }
    }
}

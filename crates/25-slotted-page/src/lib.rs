// ============================================================
//  YOUR CHALLENGE - implement a slotted page: the fundamental
//  on-disk storage unit of every relational database.
//
//  Layout: [num_slots u16][free_ptr u16][slot array...][gap][data grows down]
//  - Slot array grows forward from offset 4.
//  - Data region grows backward from free_ptr toward the slot array.
//  - Each slot entry: (offset u16, length u16). length == 0 means deleted.
//  - compact() rewrites live records contiguously to reclaim fragmented space.
// ============================================================

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
    pub fn new() -> Self { todo!() }

    fn num_slots(&self) -> usize { todo!() }
    fn free_ptr(&self) -> usize { todo!() }
    fn set_num_slots(&mut self, n: usize) { todo!() }
    fn set_free_ptr(&mut self, p: usize) { todo!() }
    fn read_slot(&self, id: SlotId) -> (usize, usize) { todo!() }
    fn write_slot(&mut self, id: SlotId, off: usize, len: usize) { todo!() }

    /// Bytes available for new record data (excluding a potential new slot entry).
    pub fn free_space(&self) -> usize { todo!() }

    /// Insert `record` and return its SlotId.
    /// Reuses a deleted slot if one exists; otherwise appends a new slot entry.
    pub fn insert(&mut self, record: &[u8]) -> Result<SlotId, PageError> { todo!() }

    /// Read a record by SlotId.
    pub fn get(&self, id: SlotId) -> Result<&[u8], PageError> { todo!() }

    /// Mark a slot as deleted (length = 0). Space is not reclaimed until compact().
    pub fn delete(&mut self, id: SlotId) -> Result<(), PageError> { todo!() }

    /// Defragment: rewrite all live records contiguously, reclaiming deleted space.
    /// Slot IDs of live records are preserved.
    pub fn compact(&mut self) { todo!() }
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
            assert_eq!(id0, id1);
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
            p.compact();
            assert!(p.insert(&big).is_ok());
        }
    }
}

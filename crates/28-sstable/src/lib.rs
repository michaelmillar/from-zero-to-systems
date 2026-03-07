// ============================================================
//  YOUR CHALLENGE - implement a Sorted String Table (SSTable).
//
//  Binary format:
//    [data section: sorted KV pairs, each: key_len u32, key, val_len u32, val]
//    [index section: every 4th entry as (key_len u32, key, offset u64)]
//    [footer: index_offset u64, index_len u64, num_entries u64]  <- 24 bytes
//
//  Writer: BTreeMap ensures sorted order. finish() writes data then index then footer.
//  Reader: open() reads footer, then index into Vec. get() binary-searches index
//          then linear-scans the block. scan() seeks to block start and collects range.
// ============================================================

use std::io::{self, Read, Seek, SeekFrom, Write};
use std::collections::BTreeMap;

const INDEX_STRIDE: usize = 4;
const FOOTER_SIZE: u64 = 24;

fn write_u32(w: &mut impl Write, v: u32) -> io::Result<()> { todo!() }
fn write_u64(w: &mut impl Write, v: u64) -> io::Result<()> { todo!() }
fn read_u32(r: &mut impl Read) -> io::Result<u32> { todo!() }
fn read_u64(r: &mut impl Read) -> io::Result<u64> { todo!() }

/// Build and write a sorted SSTable to any `Write + Seek`.
pub struct SSTableWriter {
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl SSTableWriter {
    pub fn new() -> Self { todo!() }

    pub fn add(&mut self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) { todo!() }

    /// Serialise to `dest`. Returns the byte length of the written table.
    pub fn finish<W: Write + Seek>(&self, dest: &mut W) -> io::Result<u64> { todo!() }
}

impl Default for SSTableWriter {
    fn default() -> Self { Self::new() }
}

/// Read an SSTable from any `Read + Seek`.
pub struct SSTableReader<S: Read + Seek> {
    inner: S,
    index: Vec<(Vec<u8>, u64)>,
    num_entries: u64,
    index_section_start: u64,
}

impl<S: Read + Seek> SSTableReader<S> {
    pub fn open(inner: S) -> io::Result<Self> { todo!() }

    pub fn num_entries(&self) -> u64 { todo!() }

    /// Look up a key. O(log n) via sparse index + linear scan within block.
    pub fn get(&mut self, key: &[u8]) -> io::Result<Option<Vec<u8>>> { todo!() }

    fn scan_block_for(&mut self, key: &[u8], end: u64) -> io::Result<Option<Vec<u8>>> { todo!() }

    /// Scan all entries in key order within [from, to].
    pub fn scan(&mut self, from: &[u8], to: &[u8]) -> io::Result<Vec<(Vec<u8>, Vec<u8>)>> { todo!() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn build_table(entries: &[(&str, &str)]) -> Cursor<Vec<u8>> {
        let mut w = SSTableWriter::new();
        for (k, v) in entries {
            w.add(*k, *v);
        }
        let mut buf = Cursor::new(Vec::new());
        w.finish(&mut buf).unwrap();
        buf.seek(SeekFrom::Start(0)).unwrap();
        buf
    }

    mod write_read {
        use super::*;

        #[test]
        fn write_then_read_single_entry() {
            let buf = build_table(&[("hello", "world")]);
            let mut r = SSTableReader::open(buf).unwrap();
            assert_eq!(r.get(b"hello").unwrap(), Some(b"world".to_vec()));
        }

        #[test]
        fn missing_key_returns_none() {
            let buf = build_table(&[("a", "1"), ("b", "2")]);
            let mut r = SSTableReader::open(buf).unwrap();
            assert_eq!(r.get(b"z").unwrap(), None);
        }

        #[test]
        fn entries_are_sorted_on_write() {
            let buf = build_table(&[("c", "3"), ("a", "1"), ("b", "2")]);
            let mut r = SSTableReader::open(buf).unwrap();
            assert_eq!(r.get(b"a").unwrap(), Some(b"1".to_vec()));
            assert_eq!(r.get(b"b").unwrap(), Some(b"2".to_vec()));
            assert_eq!(r.get(b"c").unwrap(), Some(b"3".to_vec()));
        }

        #[test]
        fn num_entries_is_correct() {
            let buf = build_table(&[("x", "1"), ("y", "2"), ("z", "3")]);
            let r = SSTableReader::open(buf).unwrap();
            assert_eq!(r.num_entries(), 3);
        }
    }

    mod scan {
        use super::*;

        #[test]
        fn scan_returns_entries_in_range() {
            let entries: Vec<_> = (b'a'..=b'j')
                .map(|c| (std::str::from_utf8(&[c]).unwrap().to_string(), c.to_string()))
                .collect();
            let entry_refs: Vec<_> = entries.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            let buf = build_table(&entry_refs);
            let mut r = SSTableReader::open(buf).unwrap();
            let result = r.scan(b"c", b"f").unwrap();
            let keys: Vec<_> = result.iter().map(|(k, _)| String::from_utf8(k.clone()).unwrap()).collect();
            assert_eq!(keys, vec!["c", "d", "e", "f"]);
        }

        #[test]
        fn scan_with_no_matches_returns_empty() {
            let buf = build_table(&[("a", "1"), ("b", "2")]);
            let mut r = SSTableReader::open(buf).unwrap();
            assert!(r.scan(b"x", b"z").unwrap().is_empty());
        }
    }

    mod large {
        use super::*;

        #[test]
        fn many_entries_all_retrievable() {
            let mut w = SSTableWriter::new();
            for i in 0u32..200 {
                w.add(format!("key{:04}", i), format!("val{}", i));
            }
            let mut buf = Cursor::new(Vec::new());
            w.finish(&mut buf).unwrap();
            buf.seek(SeekFrom::Start(0)).unwrap();
            let mut r = SSTableReader::open(buf).unwrap();
            for i in 0u32..200 {
                let k = format!("key{:04}", i);
                let v = format!("val{}", i);
                assert_eq!(r.get(k.as_bytes()).unwrap(), Some(v.into_bytes()), "missing {k}");
            }
        }
    }
}

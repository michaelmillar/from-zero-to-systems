use std::io::{self, Read, Seek, SeekFrom, Write};
use std::collections::BTreeMap;

const INDEX_STRIDE: usize = 4; // one index entry per N data entries
const FOOTER_SIZE: u64 = 24;   // 3 x u64

fn write_u32(w: &mut impl Write, v: u32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}
fn write_u64(w: &mut impl Write, v: u64) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}
fn read_u32(r: &mut impl Read) -> io::Result<u32> {
    let mut b = [0u8; 4]; r.read_exact(&mut b)?; Ok(u32::from_le_bytes(b))
}
fn read_u64(r: &mut impl Read) -> io::Result<u64> {
    let mut b = [0u8; 8]; r.read_exact(&mut b)?; Ok(u64::from_le_bytes(b))
}

/// Build and write a sorted SSTable to any `Write + Seek`.
pub struct SSTableWriter {
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl SSTableWriter {
    pub fn new() -> Self { Self { entries: BTreeMap::new() } }

    pub fn add(&mut self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) {
        self.entries.insert(key.into(), value.into());
    }

    /// Serialise to `dest`. Returns the byte length of the written table.
    pub fn finish<W: Write + Seek>(&self, dest: &mut W) -> io::Result<u64> {
        let data_start = dest.stream_position()?;
        let mut index: Vec<(Vec<u8>, u64)> = Vec::new();
        let mut entry_num = 0usize;

        for (key, value) in &self.entries {
            let offset = dest.stream_position()?;
            if entry_num % INDEX_STRIDE == 0 {
                index.push((key.clone(), offset));
            }
            write_u32(dest, key.len() as u32)?;
            dest.write_all(key)?;
            write_u32(dest, value.len() as u32)?;
            dest.write_all(value)?;
            entry_num += 1;
        }

        let index_offset = dest.stream_position()?;
        for (key, offset) in &index {
            write_u32(dest, key.len() as u32)?;
            dest.write_all(key)?;
            write_u64(dest, *offset)?;
        }
        let index_end = dest.stream_position()?;
        let index_len = index_end - index_offset;

        write_u64(dest, index_offset)?;
        write_u64(dest, index_len)?;
        write_u64(dest, self.entries.len() as u64)?;

        Ok(dest.stream_position()? - data_start)
    }
}

impl Default for SSTableWriter {
    fn default() -> Self { Self::new() }
}

/// Read an SSTable from any `Read + Seek`.
pub struct SSTableReader<S: Read + Seek> {
    inner: S,
    index: Vec<(Vec<u8>, u64)>, // sparse index loaded at open
    num_entries: u64,
    index_section_start: u64,
}

impl<S: Read + Seek> SSTableReader<S> {
    pub fn open(mut inner: S) -> io::Result<Self> {
        // Read footer
        inner.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))?;
        let index_offset = read_u64(&mut inner)?;
        let index_len = read_u64(&mut inner)?;
        let num_entries = read_u64(&mut inner)?;

        // Read index
        inner.seek(SeekFrom::Start(index_offset))?;
        let mut index = Vec::new();
        let mut read = 0u64;
        while read < index_len {
            let klen = read_u32(&mut inner)? as usize;
            let mut key = vec![0u8; klen];
            inner.read_exact(&mut key)?;
            let offset = read_u64(&mut inner)?;
            index.push((key, offset));
            read += 4 + klen as u64 + 8;
        }

        Ok(Self { inner, index, num_entries, index_section_start: index_offset })
    }

    pub fn num_entries(&self) -> u64 { self.num_entries }

    /// Look up a key. O(log n) via sparse index + linear scan within block.
    pub fn get(&mut self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        if self.index.is_empty() {
            // No index entries means empty table or single block starting at 0
            self.inner.seek(SeekFrom::Start(0))?;
            return self.scan_block_for(key, self.index_section_start);
        }

        // Find the last index entry with key <= target
        let start_offset = match self.index.binary_search_by(|(k, _)| k.as_slice().cmp(key)) {
            Ok(i) => self.index[i].1,
            Err(0) => {
                // Key is before the first index entry; scan from the beginning
                self.inner.seek(SeekFrom::Start(0))?;
                return self.scan_block_for(key, self.index[0].1);
            }
            Err(i) => self.index[i - 1].1,
        };

        // Determine scan end: next index entry offset or index section start
        let scan_end = self.index.iter()
            .find(|(_, off)| *off > start_offset)
            .map(|(_, off)| *off)
            .unwrap_or(self.index_section_start);

        self.inner.seek(SeekFrom::Start(start_offset))?;
        self.scan_block_for(key, scan_end)
    }

    fn scan_block_for(&mut self, key: &[u8], end: u64) -> io::Result<Option<Vec<u8>>> {
        loop {
            let pos = self.inner.stream_position()?;
            if end > 0 && pos >= end { break; }
            let klen = match read_u32(&mut self.inner) {
                Err(_) => break,
                Ok(v) => v as usize,
            };
            if klen == 0 { break; }
            let mut k = vec![0u8; klen];
            self.inner.read_exact(&mut k)?;
            let vlen = read_u32(&mut self.inner)? as usize;
            let mut v = vec![0u8; vlen];
            self.inner.read_exact(&mut v)?;
            if k.as_slice() == key { return Ok(Some(v)); }
            if k.as_slice() > key { break; }
        }
        Ok(None)
    }

    /// Scan all entries in key order within [from, to].
    pub fn scan(&mut self, from: &[u8], to: &[u8]) -> io::Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let start_offset = if self.index.is_empty() {
            0
        } else {
            match self.index.binary_search_by(|(k, _)| k.as_slice().cmp(from)) {
                Ok(i) => self.index[i].1,
                Err(0) => 0,
                Err(i) => self.index[i - 1].1,
            }
        };

        self.inner.seek(SeekFrom::Start(start_offset))?;
        let mut out = Vec::new();
        loop {
            let pos = self.inner.stream_position()?;
            if self.index_section_start > 0 && pos >= self.index_section_start { break; }
            let klen = match read_u32(&mut self.inner) {
                Err(_) => break,
                Ok(v) => v as usize,
            };
            if klen == 0 { break; }
            let mut k = vec![0u8; klen];
            self.inner.read_exact(&mut k)?;
            let vlen = read_u32(&mut self.inner)? as usize;
            let mut v = vec![0u8; vlen];
            self.inner.read_exact(&mut v)?;
            if k.as_slice() > to { break; }
            if k.as_slice() >= from { out.push((k, v)); }
        }
        Ok(out)
    }
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
            // Insert out of order; reader must return sorted
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

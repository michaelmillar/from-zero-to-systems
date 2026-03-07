# sstable

> SSTable -- sorted, immutable key-value file with sparse index and range scan.

## ELI5

An SSTable is like a phone book: entries are sorted alphabetically, you can't change it once printed, and the index at the front tells you roughly where to look. To find "Smith", you open near the S section and scan forward. To find everyone between "Smith" and "Taylor", you just read that section. No random access needed.

## For the Educated Generalist

An **SSTable (Sorted String Table)** is the on-disk component of an LSM-tree (Log-Structured Merge-tree). It has three properties that define it:

1. **Sorted**: entries are ordered by key, so binary search and range scans are efficient
2. **Immutable**: once written, never modified. Updates go to a new SSTable; old ones are compacted away
3. **Indexed**: a sparse in-memory index points to block boundaries so you don't scan from the start

**On-disk format** (this implementation):

```
[data section]
  [key_len: u32][key bytes][val_len: u32][val bytes]  ...  (sorted)
[index section]
  [key_len: u32][key bytes][offset: u64]  ...  (every 4th entry)
[footer: 24 bytes]
  [index_offset: u64][index_len: u64][num_entries: u64]
```

**Lookup** (`get`):
1. Binary search the sparse index to find the last index entry with key <= target
2. Seek to that offset and scan forward linearly until key found or key > target

**Range scan** (`scan`):
1. Same index lookup to find the start block
2. Linear scan from there, collecting entries in [from, to]

**Immutability's advantages**: no locking needed (readers and writers don't interfere), files can be memory-mapped, and old versions are just file deletions. The cost: you can't update a value in place -- you must write a new file and eventually compact.

The `SSTableWriter` uses a `BTreeMap` to sort entries at write time. The `SSTableReader` loads only the sparse index into memory (a few KB for millions of entries), keeping the memory footprint tiny.

## Used in the wild

- **LevelDB / RocksDB** -- the entire storage engine is a hierarchy of SSTables with periodic compaction
- **Apache Cassandra** -- uses SSTables for its SSTable-backed LSM engine; `nodetool compact` merges them
- **ClickHouse** -- columnar storage uses a similar sorted, immutable file format with sparse indexing
- **BigTable (Google)** -- the original paper that described SSTables as the persistent layer of an LSM-tree

## Run it

```bash
cargo run -p sstable
```

## Rust concepts covered

- **`BTreeMap` for sorted writes**: Rust's `BTreeMap` keeps keys sorted; iterating it in order costs nothing extra
- **`Read + Seek` trait bounds**: the `SSTableReader` is generic over any seekable byte source -- works with `File`, `Cursor<Vec<u8>>`, memory-mapped files, or network streams
- **`std::io::Cursor<Vec<u8>>`**: wraps an in-memory buffer as a `Read + Write + Seek` -- ideal for testing I/O code without files
- **Little-endian framing**: length-prefixed records with explicit byte-order encoding ensure cross-platform compatibility

## Builds on

Standalone -- no earlier crates required. In a full LSM-tree, the skip list (crate 29) acts as the in-memory MemTable that flushes to an SSTable when full.

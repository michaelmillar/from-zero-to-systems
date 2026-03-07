# slotted-page

> Slotted-page storage format -- variable-length records, compaction, and free-space management.

## ELI5

Imagine a page in a notebook where you write sentences from the bottom up, and keep an index at the top that tells you where each sentence starts. If you cross out a sentence, the space is wasted until you rewrite the whole page -- but you don't have to renumber everything you kept. That's a slotted page. Every database (Postgres, SQLite, MySQL) stores table rows this way.

## For the Educated Generalist

A **slotted page** is a fixed-size block (typically 4 KB -- matching the OS memory page size) with a two-region layout:

```
[header: num_slots u16, free_ptr u16]
[slot array grows downward: (offset u16, length u16) per slot]
        ... free space gap ...
[record data grows upward from the bottom of the page]
```

Key properties:

- **Variable-length records**: each record can be any size; only `(offset, length)` is stored in the slot array
- **Stable slot IDs**: deleting record 1 does not renumber record 2; the slot ID is a permanent reference (like a row ID in a database)
- **Lazy compaction**: deletion marks the slot length as 0; space is only physically reclaimed when `compact()` is called, which rewrites live records contiguously and updates their offsets
- **Free space tracking**: `free_ptr` marks the boundary between the header/slot region and the data region; inserting checks that `free_ptr - slot_end >= record_len + slot_entry_size`

This layout is used in virtually every page-oriented storage engine (Postgres heap files, SQLite B-tree pages, InnoDB pages) because it is cache-friendly, supports variable-length data without external fragmentation, and keeps row lookups O(1).

## Used in the wild

- **PostgreSQL** -- heap files store tuples (rows) in slotted pages; the `ctid` system column is literally `(page_number, slot_id)`
- **SQLite** -- B-tree pages use a similar slot-offset scheme for both table and index pages
- **InnoDB (MySQL)** -- each 16 KB page has a directory of slots pointing to row records
- **RocksDB** -- SSTable data blocks use a prefix-compressed variant of this layout for sorted key-value pairs

## Run it

```bash
cargo run -p slotted-page
```

## Rust concepts covered

- **`Vec<u8>` as a byte buffer**: treating raw bytes as a typed structure by reading/writing multi-byte integers manually with `to_le_bytes` / `from_le_bytes`
- **Little-endian encoding**: why databases use fixed endianness (portability, on-disk format stability)
- **`saturating_sub`**: preventing underflow when computing free space across unsigned arithmetic
- **`filter_map` + `collect`**: collecting live records during compaction without allocating intermediate structures

## Builds on

Standalone -- no earlier crates required. Crates 26 (b-tree) and 27 (buffer-pool) both depend on this crate.

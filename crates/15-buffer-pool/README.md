# buffer-pool

> Database buffer pool -- clock eviction, pin/unpin, dirty page tracking.

## ELI5

A database stores data on disk, but disk is slow. The buffer pool is the database's private RAM cache: it holds recently used pages so reads don't always go to disk. When the cache is full and a new page is needed, the buffer pool must evict an old page -- but only if no one is still using it. If a page was modified, it must be written back to disk first.

## For the Educated Generalist

The buffer pool is the single most important component in a database engine's I/O path. Its job is to maintain a fixed pool of in-memory **frames**, each holding one disk page, and to manage which pages are in memory at any time.

**Core operations:**

- `pin(page_id)` -- bring a page into a frame (fetching from disk if needed), increment its pin count, return a mutable reference. A pinned page cannot be evicted.
- `unpin(page_id, is_dirty)` -- release the caller's hold on the page. If `is_dirty = true`, the frame is marked dirty and must be written back before eviction.
- `flush_dirty()` -- write all dirty, unpinned frames to disk.

**Clock eviction** (second-chance algorithm): the clock hand sweeps frames in a circle. If a frame's `ref_bit` is set (it was recently accessed), clear the bit and give it a second chance. If the `ref_bit` is already clear and the frame is unpinned, evict it. If the frame is pinned, skip it. The clock is O(capacity) in the worst case but O(1) amortised, and approximates LRU without the overhead of maintaining a linked list.

**Why not LRU?** True LRU requires updating a linked list on every access -- that's a global lock on every read in a multi-threaded database. Clock is a lock-free approximation that performs almost as well in practice.

**Pin count** tracks concurrent readers/writers. A page with `pin_count > 0` is "in use" and cannot be evicted, even if the eviction algorithm selects it. This is the buffer pool's version of reference counting.

## Used in the wild

- **PostgreSQL** -- `shared_buffers` is a tunable buffer pool. Sized at 25% of RAM by default.
- **InnoDB (MySQL)** -- `innodb_buffer_pool_size` is the single most impactful tuning parameter.
- **SQLite** -- the page cache uses a similar pin/unpin model, though simpler (single-process).
- **RocksDB** -- the Block Cache is a buffer pool for SSTable data blocks.

## Run it

```bash
cargo run -p buffer-pool
```

## Rust concepts covered

- **`HashMap<PageId, usize>`** as a page table: O(1) lookup of which frame holds a given page
- **`Vec<Frame>` with index-based access**: avoids self-referential lifetimes; frames are identified by index, not pointer
- **Mutable return from a method**: `pin` returns `&mut Vec<u8>` into `self.frames[idx].data` -- the borrow checker ensures the reference is valid for the duration of use
- **Simulated I/O**: `Disk` uses a `Vec<Vec<u8>>` to make the buffer pool fully testable without real file I/O

## Builds on

- [`slotted-page`](../25-slotted-page/) -- pages managed by the buffer pool contain slotted-page formatted data. The two crates together form the storage layer: buffer pool handles WHICH page is in memory; slotted page handles WHAT is inside a page.

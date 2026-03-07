# skip-list

> Skip list -- probabilistic sorted structure, the canonical LSM-tree MemTable.

## ELI5

A linked list lets you insert anywhere in O(1), but finding things is slow (O(n) -- you scan every node). A sorted array is fast to search (binary search, O(log n)) but slow to insert. A skip list gets you both: O(log n) insert AND O(log n) lookup, by building a hierarchy of "express lanes" on top of a linked list, where each express lane skips over more and more nodes. The number of lanes a node gets is decided by a coin flip.

## For the Educated Generalist

A **skip list** maintains multiple linked lists at different "levels". Level 0 is the complete sorted list. Level 1 connects every ~2nd node (on average). Level 2 connects every ~4th node. And so on, up to `MAX_LEVEL`.

**Search**: start at the highest level. Walk forward until the next key would overshoot the target, then drop down a level and repeat. This converges to the target in O(log n) steps -- the same asymptotic complexity as a balanced BST, but without the rebalancing machinery.

**Insert**: follow the same search path, collecting the predecessor at each level (`update` array). Generate a random level for the new node (each level above 1 added with probability 0.5). Link the new node in at all its levels by adjusting the predecessor pointers.

**Probabilistic balance**: a B-tree or AVL tree must explicitly rebalance after every insert. A skip list relies on probability: the expected structure is balanced without any explicit rotation or split logic. The expected height of any node is O(log n), and the probability of a pathologically tall node decreases exponentially.

**Memory ownership in unsafe Rust**: each node is heap-allocated via `Box::into_raw`. Ownership flows exclusively through the level-0 chain -- the `Drop` impl walks it to reclaim each node exactly once. Higher-level pointers are raw, non-owning aliases. The `node_ref` helper converts raw pointers to explicit shared references, avoiding the `dangerous_implicit_autorefs` lint.

## Used in the wild

- **RocksDB / LevelDB** -- the in-memory MemTable is a skip list. Every write goes here first; when full, the skip list is flushed to an SSTable (crate 28).
- **Redis** -- sorted sets (`ZADD`, `ZRANGE`) are implemented as skip lists
- **Apache Cassandra** -- memtable is a concurrent skip list (using `ConcurrentSkipListMap` from Java's `java.util.concurrent`)
- **CockroachDB** -- uses a skip list for in-memory interval trees

## Run it

```bash
cargo run -p skip-list
```

## Rust concepts covered

- **`Box::into_raw` / `Box::from_raw`**: transferring heap ownership to and from raw pointers -- the mechanism for manually managed memory in Rust
- **`unsafe impl Send`**: explicitly asserting thread safety for a data structure with raw pointers, backed by a documented ownership model
- **Raw pointer aliasing rules**: multiple `*mut` pointers alias the same nodes at different levels; only level-0 is owning; all others are observing. This is sound because the owner (level-0 chain) always outlives the observers.
- **`thread_local!` with `Cell<u64>`**: cheap, seedable PRNG without heap allocation or synchronisation -- correct for per-thread use in the skip list's level generator

## Builds on

Standalone -- no earlier crates required. Pair with crate 28 (sstable) to see the full MemTable-to-SSTable flush pipeline of an LSM-tree.

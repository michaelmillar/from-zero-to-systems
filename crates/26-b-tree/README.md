# b-tree

> B-tree index -- insert, lookup, range scan, with automatic node splitting.

## ELI5

A B-tree is like an organised filing cabinet where every drawer is kept at least half-full, and the labels between drawers tell you which one to look in. Because every node is always roughly the same size, looking something up takes the same number of steps whether you have 100 records or 100 million. That predictability is why every database index is a B-tree.

## For the Educated Generalist

A **B-tree of order 5** means each internal node holds up to 4 separator keys and 5 child pointers. When a node overflows (exceeds 4 keys), it splits in half and pushes the median key up to the parent. If the root splits, a new root is created -- this is the only way the tree grows taller.

Properties that matter for databases:

- **Guaranteed height**: O(log_B n) levels. For a million rows and order 500 (real databases use large pages), the tree is 3-4 levels deep -- roughly 3-4 I/Os to find any row.
- **Sorted order**: in-order traversal of leaves gives all keys sorted. This makes range queries natural and cheap.
- **All data in leaves**: this implementation stores values in leaves only (like a B+ tree). Internal nodes hold only separator keys, so non-leaf pages can hold more keys, reducing height.

**Node splitting** is the core operation. When a leaf exceeds `MAX_KEYS`, it splits at the midpoint: the right half becomes a new sibling node, and the first key of the right half "rises" to the parent as a separator. The same applies recursively to internal nodes. The `split_internal` function pops the median key out (it goes up to the parent) and the right children go with the right node.

**Range scan** prunes subtrees using separator keys: if the separator below a child's range is already > `to`, skip that child entirely.

## Used in the wild

- **PostgreSQL / MySQL / SQLite** -- every table index uses a B-tree (or B+ tree variant)
- **Filesystems** -- HFS+, ext4, NTFS, Btrfs all use B-trees for directory entries and extent maps
- **InnoDB** -- the clustered index IS the table; rows are stored in leaf pages of a B+ tree keyed by primary key

## Run it

```bash
cargo run -p b-tree
```

## Rust concepts covered

- **Recursive enums**: `Node::Leaf | Node::Internal` with `Box<Node>` children -- a textbook recursive type
- **`partition_point`**: binary search variant that returns the insertion point; cleaner than manual binary search
- **`Vec::split_off`**: splits a Vec at an index, taking the tail into a new Vec -- perfect for node splitting
- **`std::mem::replace`**: swaps a value out of a struct field without cloning -- used to replace the root during root splits

## Builds on

- [`slotted-page`](../25-slotted-page/) -- in a disk-backed B-tree, each `Node` would be serialised into a slotted page and managed by a buffer pool. This crate implements the in-memory tree logic; the README connects the pieces.

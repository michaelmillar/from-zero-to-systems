# hash-table

> Hash table from scratch -- open addressing with Robin Hood probing and dynamic resize.

## ELI5

A hash table is like a filing cabinet where you invent your own numbering system. When you want to file something, you run the label through a formula that gives you a drawer number. Later, you run the same formula on the label to instantly find it again. The challenge is: what if two labels map to the same drawer? That's the "collision" problem, and Robin Hood hashing is one elegant solution.

## For the Educated Generalist

This implements a **hash table with open addressing** and **Robin Hood probing**.

In open addressing, all entries live in a single flat array (no linked lists, no boxing). When a collision occurs, we probe forward until an empty slot is found. The probe sequence here is linear probing (idx, idx+1, idx+2...).

**Robin Hood hashing** adds an invariant: no entry may have a *shorter* probe distance than an entry that displaced it. On insertion, if the new entry's probe distance exceeds the occupant's, we *steal the slot* and continue inserting the displaced entry. This reduces variance in probe lengths -- the maximum probe distance grows as O(log n) rather than O(n) in the worst case.

**Deletion** uses backward shifting: rather than leaving a tombstone (which pollutes future lookups), we shift subsequent entries one slot backward, maintaining the Robin Hood invariant.

**Resize** triggers at 70% load factor, doubling capacity and rehashing all entries.

The lookup early-exit rule: during a search, if our probe distance exceeds the distance of the current occupant, the key cannot be present (by the Robin Hood invariant). This allows O(1) average negative lookups.

## Used in the wild

- **Every programming language runtime** -- Python `dict`, Java `HashMap`, Rust `HashMap` all use open addressing variants
- **Database hash joins** -- the probe side of a hash join builds an in-memory table exactly like this
- **Compiler symbol tables** -- variable and function name lookup during parsing and type-checking

## Run it

```bash
cargo run -p hash-table
```

## Rust concepts covered

- **Generics with trait bounds**: `K: Eq + Hash + Clone` -- requiring exactly the capabilities needed, nothing more
- **`std::hash::{Hash, Hasher, DefaultHasher}`**: the standard hashing infrastructure
- **`std::mem::replace`**: swap a value out of a data structure without cloning
- **Enums as tagged unions**: `Slot::Empty | Slot::Occupied { .. }` -- zero-overhead state machine per slot

## Builds on

Standalone -- no earlier crates required.

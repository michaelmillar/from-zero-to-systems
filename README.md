# from-zero-to-systems

![crate map](map.svg)

> Build increasingly complex Rust applications, from probability engines to distributed consensus, grounded in real-world use cases across finance, science, infrastructure, AI, and security.

## Who this is for

Developers coming from Python, Go, TypeScript, or another language who already understand programming fundamentals and want to learn Rust by building real things. No toy exercises. Every crate is a working application with a genuine use case.

## How it works

Each numbered crate under `crates/` is:

- **Independently runnable:** `cargo run -p <crate-name>`
- **A reusable library:** later crates import earlier ones as dependencies
- **Self-documenting:** each README has an ELI5, an educated-generalist explanation, real-world "used in the wild" callouts, and Rust concepts covered

## Dependency Graph

```
01-risk-sampler ──────────────────────────────────────────────── standalone
02-probability-engine ──── depends on ──► 01
03-monte-carlo ─────────── depends on ──► 02
04-distribution-sampler ── depends on ──► 02
05-statistics-core ─────── depends on ──► 03
06-matrix-math ────────────────────────────────────────────────── standalone
07-linear-regression ───── depends on ──► 05, 06
08-signal-processing ───────────────────────────────────────────── standalone
09-bit-manipulator ────────────────────────────────────────────── standalone
10-memory-arena ───────────────────────────────────────────────── standalone
11-float-inspector ─────────────────────────────────────────────── standalone
12-mini-vm ─────────────── depends on ──► 09, 10
13-slotted-page ────────────────────────── standalone
14-b-tree ──────────────── depends on ──► 13
15-buffer-pool ─────────── depends on ──► 13
16-sstable ─────────────────────────────── standalone
17-skip-list ───────────────────────────── standalone
18-consistent-hashing ─────────────────────────────────────────── standalone
19-bloom-filter ───────────────────────────────────────────────── standalone
20-rate-limiter ───────────────────────────────────────────────── standalone
21-merkle-tree ─────────── depends on ──► 19
22-gossip-protocol ─────── depends on ──► 20
23-raft-consensus ──────── depends on ──► 22
24-gradient-descent ────── depends on ──► 06
25-neural-net ──────────── depends on ──► 06, 24
26-decision-tree ───────── depends on ──► 05, 25
27-k-means ─────────────── depends on ──► 05, 06
28-attention-mechanism ─── depends on ──► 06, 24
29-bpe-tokeniser ───────── depends on ──► 09
```

## Playing the game

The tests are the spec. Your job is to make them pass.

The reference implementations live in this repo. Fork it or create a solutions worktree, clear each crate's implementation, and rebuild it from scratch.

**Start the game runner**

```bash
cargo run -p play
```

```
  from-zero-to-systems                                          01 / 29
─────────────────────────────────────────────────────────────────────────
  01  02  03  04  05  06  07  08  09  10  11  12  13  14  ...
  .   .   .   .   .   .   .   .   .   .   .   .   .   .   ...
─────────────────────────────────────────────────────────────────────────
  01 . risk-sampler                       | info
                                          |
  press r to run tests                    | Simulate risk events across
                                          | thousands of trials to
                                          | calculate Value at Risk.
                                          |
                                          | Completed  0 / 29
─────────────────────────────────────────────────────────────────────────
  r run  .  h hint  .  d docs  .  c concepts  .  <- -> navigate  .  q quit
```

**1. Clear the implementation**

Open `crates/01-risk-sampler/src/lib.rs`. Delete everything *above* the `#[cfg(test)]` block -- the structs, function, all of it. Leave the tests completely untouched. Press **r** in the runner:

```
  from-zero-to-systems                                          01 / 29
─────────────────────────────────────────────────────────────────────────
  01  02  03  04  05  06  07  08  09  10  11  12  13  14  ...
  x   .   .   .   .   .   .   .   .   .   .   .   .   .   ...
─────────────────────────────────────────────────────────────────────────
  01 . risk-sampler                       | info
                                          |
  x  zero_probability_event_never_occurs  | Completed  0 / 29
  x  certain_event_always_occurs          |
  x  var_95_is_not_greater_than_max_loss  |
  x  mean_loss_is_consistent_with_proba.. |
─────────────────────────────────────────────────────────────────────────
  r run  .  h hint  .  d docs  .  c concepts  .  <- -> navigate  .  q quit
```

Four failing tests. That is your todo list.

**2. Read the first test -- it tells you exactly what to build**

```rust
#[test]
fn zero_probability_event_never_occurs() {
    let events = vec![RiskEvent {
        name: "never".into(),
        probability: 0.0,
        max_loss: 1_000_000.0,
    }];
    let result = simulate(&events, 10_000, 42);
    assert_eq!(result.occurrences, 0);
    assert_eq!(result.total_loss, 0.0);
}
```

It needs a `RiskEvent` struct, a `SimulationResult` struct, and a `simulate` function. Add the minimum to make this compile and pass. Press **h** for a nudge if you are stuck.

**3. Press r -- watch it go green**

```
  from-zero-to-systems                                          01 / 29
─────────────────────────────────────────────────────────────────────────
  01  02  03  04  05  06  07  08  09  10  11  12  13  14  ...
  v   .   .   .   .   .   .   .   .   .   .   .   .   .   ...
─────────────────────────────────────────────────────────────────────────
  01 . risk-sampler                       | info
                                          |
  v  zero_probability_event_never_occurs  | Completed  1 / 29
  v  certain_event_always_occurs          |
  v  var_95_is_not_greater_than_max_loss  |
  v  mean_loss_is_consistent_with_proba.. |
─────────────────────────────────────────────────────────────────────────
  r run  .  h hint  .  d docs  .  c concepts  .  <- -> navigate  .  q quit
```

Press **n** to move to crate 02 and repeat. Work in order -- later crates import earlier ones.

**Install the short command**

```bash
cargo install --path play --bin fzts --force
```

**Play on the web**

```bash
fzts
```

Opens `http://127.0.0.1:7878/web/index.html` with a three-column workspace: module navigation and briefing on the left, code editor in the centre, concepts and test output on the right. Challenge 01 ships in Rust, C, Python, and Haskell; switch languages from the toolbar dropdown.

![web ui](assets/web-ui.png)

On first launch `fzts` creates a local `.fzts/workspace` seeded from the sibling `from-zero-to-systems-challenges` worktree. Your progress and edits stay local (`.fzts/` is gitignored).

## Tiers

| Tier | Crates | Domain |
|------|--------|--------|
| 1 | 01-05 | Probability & Statistics |
| 2 | 06-08 | Linear Algebra |
| 3 | 09-12 | Low-Level Systems |
| 4 | 13-17 | Storage Internals |
| 5 | 18-23 | Distributed Systems |
| 6 | 24-29 | AI & Machine Learning |

## Licence

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

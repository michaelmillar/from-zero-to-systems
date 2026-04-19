<h3 align="center">from-zero-to-systems</h3>

<p align="center">
  From first crate to multi-crate system. Learn Rust through codebase evolution.<br>
  Each crate is a working system. Later crates depend on earlier ones. At arc boundaries you refactor, benchmark, and maintain what you built.
</p>

---

https://github.com/user-attachments/assets/6ba74b3f-ce66-4738-b5bb-c8e7f1fc487a

---

<p align="center">
  <img src="map.svg" alt="crate dependency map" width="920" />
</p>

## What it does

A graded sequence of Rust crates forming a single workspace. Each crate is independently runnable and solves a real problem. Later crates import earlier ones as library dependencies, so the complexity compounds naturally.

At the end of each arc you return to earlier crates for evolution milestones. Refactor a public API without breaking dependents. Benchmark with criterion. Audit unsafe code. Add tracing. Inject failures. Profile memory. This is the maintenance work that separates production Rust from tutorial Rust.

A TUI runner and browser workspace track your progress.

```
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

## Who this is for

Experienced developers coming from Python, Go, TypeScript, or another language who already understand programming fundamentals and want to think in Rust systems. If you have built production software in another language and want to understand ownership, lifetimes, traits, and unsafe in the context of growing and maintaining a real codebase, start here.

## How it works

1. Clear the implementation in `crates/01-risk-sampler/src/lib.rs`. Leave the tests untouched.
2. Read the first test. It tells you exactly what to build.

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

3. Write the minimum to make it pass. Press `r` in the runner. Watch it go green.
4. Move to the next crate. Later crates import earlier ones.
5. At each arc boundary, complete the evolution milestone before moving on.

## Web UI

```
cargo install --path play --bin fzts --force
fzts
```

Opens `http://127.0.0.1:7878/web/index.html` with a three-column workspace. Challenge 01 ships in Rust, C, Python, and Haskell.

## Arcs

| Arc | Name | Domain | Crates | Evolution milestone |
|-----|------|--------|--------|---------------------|
| 1 | Chance | Probability and Statistics | 01 to 05 | Refactor a public API without breaking dependents. Add property-based tests. |
| 2 | Shape | Linear Algebra and Signals | 06 to 08 | Benchmark with criterion. Optimise a hot path. |
| 3 | Metal | Low-Level Systems | 09 to 12 | Audit unsafe, isolate behind safe wrappers, add fuzzing. |
| 4 | Durability | Storage Internals | 13 to 17 | Add tracing instrumentation. Profile memory under load. |
| 5 | Scale | Distributed Systems | 18 to 23 | Inject failures into gossip. Chaos-test Raft under partition. |
| 6 | Intelligence | AI and Machine Learning | 24 to 29 | Cross-crate performance audit. Detect and fix a regression. |

## How it compares

Most learn-Rust resources teach you to write Rust. Rustlings and Exercism are collections of isolated exercises. Zero To Production and CodeCrafters each build a single application. from-zero-to-systems is a growing multi-crate workspace where later crates depend on earlier ones, and at every arc boundary you return to maintain what you already built.

**Where this is stronger.** The dependency graph means changes ripple. Refactoring a public API in Arc 1 requires updating dependents in Arc 2. Benchmarking in Arc 2 uses the code you wrote in Arc 1. Unsafe auditing in Arc 3 forces you to wrap the arena allocator you built earlier behind a safe interface. This is how production Rust codebases actually evolve.

**Where this is weaker.** It does not teach async Rust, web frameworks, or GUI development. It is not a reference. If you want to look up how iterators work, read the Rust Book. If you want to build systems and understand why Rust makes the choices it does, start here.

## Dependency graph

```
Arc 1  Chance
─────────────
01-risk-sampler ──────────────────────────────────────────── standalone
02-probability-engine ──── depends on ──► 01
03-monte-carlo ─────────── depends on ──► 02
04-distribution-sampler ── depends on ──► 02
05-statistics-core ─────── depends on ──► 03
  >> EVOLVE  refactor public API, add property-based tests

Arc 2  Shape
────────────
06-matrix-math ────────────────────────────────────────────── standalone
07-linear-regression ───── depends on ──► 05, 06
08-signal-processing ───────────────────────────────────────── standalone
  >> EVOLVE  benchmark with criterion, optimise hot path

Arc 3  Metal
────────────
09-bit-manipulator ────────────────────────────────────────── standalone
10-memory-arena ───────────────────────────────────────────── standalone
11-float-inspector ─────────────────────────────────────────── standalone
12-mini-vm ─────────────── depends on ──► 09, 10
  >> EVOLVE  audit unsafe, isolate behind safe wrappers, fuzz

Arc 4  Durability
─────────────────
13-slotted-page ────────────────────────── standalone
14-b-tree ──────────────── depends on ──► 13
15-buffer-pool ─────────── depends on ──► 13
16-sstable ─────────────────────────────── standalone
17-skip-list ───────────────────────────── standalone
  >> EVOLVE  add tracing instrumentation, profile memory

Arc 5  Scale
────────────
18-consistent-hashing ─────────────────────────────────────── standalone
19-bloom-filter ───────────────────────────────────────────── standalone
20-rate-limiter ───────────────────────────────────────────── standalone
21-merkle-tree ─────────── depends on ──► 19
22-gossip-protocol ─────── depends on ──► 20
23-raft-consensus ──────── depends on ──► 22
  >> EVOLVE  failure injection, chaos-test Raft under partition

Arc 6  Intelligence
───────────────────
24-gradient-descent       (standalone)
25-neural-net             (standalone)
26-decision-tree          (standalone)
27-k-means                (standalone)
28-attention-mechanism    (standalone)
29-bpe-tokeniser          (standalone)
  >> EVOLVE  cross-crate performance audit, detect and fix regression
```

## Licence

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

# from-zero-to-systems

> Build increasingly complex Rust applications, from probability engines to distributed consensus, grounded in real-world use cases across finance, science, infrastructure, AI, and security.

## Who this is for

Developers coming from Python, Go, TypeScript, or another language who already understand programming fundamentals and want to learn Rust by building real things. No toy exercises. Every crate is a working application with a genuine use case.

---

## Quickstart

```bash
git clone https://github.com/michaelmillar/from-zero-to-systems
cd from-zero-to-systems
cargo build
cargo run -p play
```

That last command opens the interactive learning runner. Everything you need is in there.

---

## How to play

### The interactive runner

`cargo run -p play` launches a full-screen TUI. Run it in a terminal alongside your editor.

When you first open a crate the right panel shows what it is about:

```
 01  02  03  04  05  06  07  08  09  10  11  12 ...
  ·   ·   ·   ·   ·   ·   ·   ·   ·   ·   ·   ·
┌─ 01 · risk-sampler ──────────┐ ┌─ Info ────────────────────────────────┐
│                               │ │ Simulate risk events across thousands │
│  Press [r] to run tests       │ │ of trials to calculate Value at Risk. │
│                               │ │ The 95th-percentile trial loss is     │
│                               │ │ your VaR 95 — the figure used by      │
│                               │ │ banks and insurers to size capital.   │
│                               │ │                                       │
│                               │ │ Completed: 0/29                       │
└───────────────────────────────┘ └───────────────────────────────────────┘
  [r]un  [h]int  [d]ocs  [c]oncepts  [←/p]prev  [→/n]next  [q]uit
```

After pressing `r` to run tests and `h` on a failing test:

```
┌─ 01 · risk-sampler ──────────┐ ┌─ Hint 1/3 ────────────────────────────┐
│ ✗ zero_probability_event_... │ │ Test: zero_probability_event_never_.. │
│ ✗ certain_event_always_...   │ │                                       │
│ ✗ var_95_is_not_greater_...  │ │ Start here: let mut rng =             │
│ ✗ mean_loss_is_consistent... │ │ StdRng::seed_from_u64(seed); then     │
│                               │ │ loop for _ in 0..trials { }.         │
└───────────────────────────────┘ └───────────────────────────────────────┘
  [r]un  [h]int  [d]ocs  [c]oncepts  [←/p]prev  [→/n]next  [q]uit
```

### Keys

| Key | Action |
|-----|--------|
| `r` | Run all tests for the current crate (`cargo test -p <name>`) |
| `h` | Reveal the next hint for the selected test (up to 3 per test) |
| `d` | Show documentation links for the current crate |
| `c` | Show the Rust and systems concepts this crate teaches |
| `n` / `→` | Move to the next crate |
| `p` / `←` | Move to the previous crate |
| `j` / `↓` | Select the next test in the list |
| `k` / `↑` | Select the previous test in the list |
| `Esc` | Close the hint/docs/concepts panel |
| `q` | Quit |

The top strip shows all crates at a glance. Green `✓` means all tests pass. Red `✗` means failures. `·` means not yet run.

### Your workflow

1. **Open the runner** alongside your editor.
2. **Navigate** to crate `01-risk-sampler` with `n`/`p`.
3. **Press `r`** to run the tests. They will all fail — that is expected.
4. **Select a failing test** with `j`/`k` and read its name carefully. The test name IS the spec.
5. **Open `crates/01-risk-sampler/src/lib.rs`** in your editor and implement the function.
6. **Press `r` again** to re-run. Fix, repeat until that test goes green.
7. **Move to the next test.** Tackle them one at a time.
8. **Press `h`** if you are stuck. Hints are revealed one at a time — try to use as few as possible.
9. Once all tests pass, the runner marks the crate complete and your progress is saved.
10. **Move to the next crate** with `n`.

### If you are completely stuck

Press `h` up to three times on the selected test:

- **Hint 1** explains the structure — what shape the code should take.
- **Hint 2** explains the algorithm — what to compute.
- **Hint 3** gives exact Rust syntax — a near-complete code snippet.

Press `c` to see the Rust concepts this crate teaches. Press `d` to open documentation links.

### Without the runner

You can work entirely from the command line if you prefer:

```bash
# Run all tests for one crate
cargo test -p risk-sampler

# Run a specific test by name
cargo test -p risk-sampler zero_probability

# Run the binary to see what it does once you have it working
cargo run -p risk-sampler
```

---

## Tiers

| Tier | Crates | Domain |
|------|--------|--------|
| 1 | 01-04 | Simulation and Probability |
| 2 | 05-08 | Maths and Statistics |
| 3 | 09-12 | Low-Level Systems |
| 4 | 13-18 | Distributed Systems |
| 5 | 19-24 | AI and Machine Learning |
| 6 | 25-29 | Systems and Kernel |

---

## Dependency graph

Later crates import earlier ones — completing them in order keeps the dependency graph working.

```
01-risk-sampler ──────────────────────────────────────────────── standalone
02-probability-engine ──── depends on ──► 01
03-monte-carlo ─────────── depends on ──► 02
04-distribution-sampler ── depends on ──► 02
05-statistics-core ─────── depends on ──► 03
06-matrix-math ────────────────────────────────────────────────── standalone
07-linear-regression ───── depends on ──► 05, 06
08-signal-processing ───── depends on ──► 06
09-bit-manipulator ────────────────────────────────────────────── standalone
10-memory-arena ───────────────────────────────────────────────── standalone
11-float-inspector ─────── depends on ──► 07
12-mini-vm ─────────────── depends on ──► 09, 10
13-consistent-hashing ─────────────────────────────────────────── standalone
14-bloom-filter ───────────────────────────────────────────────── standalone
15-rate-limiter ───────────────────────────────────────────────── standalone
16-merkle-tree ─────────── depends on ──► 14
17-gossip-protocol ─────── depends on ──► 15
18-raft-consensus ──────── depends on ──► 17
19-gradient-descent ────── depends on ──► 06
20-neural-net ──────────── depends on ──► 06, 19
21-decision-tree ───────── depends on ──► 05, 20
22-k-means ─────────────── depends on ──► 05, 06
23-attention-mechanism ─── depends on ──► 06, 19
24-bpe-tokeniser ───────── depends on ──► 09
25-mmio-registers ──────────────────────────────────────────────── standalone
26-char-device-driver ─────────────────────────────────────────── standalone
27-process-scheduler ───────────────────────────────────────────── standalone
28-raw-socket ──────────────────────────────────────────────────── standalone
29-ebpf-probe ──────────────────────────────────────────────────── standalone
```

---

## Each crate contains

- `src/lib.rs` — the challenge: function stubs with `todo!()` and BDD-style tests that define the spec
- `src/main.rs` — a runnable binary showing the library working in a real scenario
- `README.md` — plain-English explanation, Rust concepts covered, and real-world "used by X" callouts

---

## Licence

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

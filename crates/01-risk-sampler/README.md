# risk-sampler

> Simulate portfolios of risk events to estimate total loss exposure and Value at Risk.

## ELI5

Imagine you run a business and three bad things might happen this year: a flood, a burglary, and a computer crash. Each one has a different chance of happening and would cost a different amount to fix. A risk sampler plays out your year thousands of times — sometimes the flood happens, sometimes it doesn't — and adds up all the costs each time. After running the simulation, you can see things like "in 95% of simulated years, we'd lose less than £80,000." That's really useful for deciding how much insurance to buy.

## For the Educated Generalist

Risk quantification is the problem of turning uncertain future events into actionable numbers. The naive approach — listing all the bad things that might happen and summing their expected costs — breaks down because it ignores correlation (multiple events hitting at once) and gives you no sense of the *distribution* of outcomes, only the mean.

Monte Carlo simulation fixes this by sampling from the joint distribution directly. For each trial, we roll the dice for every risk event independently. This lets us compute not just expected loss but **Value at Risk (VaR)** — the loss threshold you won't exceed in X% of scenarios. VaR at 95% means: "In 95 out of 100 simulated years, total losses stay below this number."

This is the same technique used by insurance actuaries, bank risk desks (under Basel III requirements), and infrastructure reliability engineers calculating SLA budgets.

## What it does

Simulates a configurable portfolio of risk events over N trials using a seeded random number generator for reproducibility. Outputs total loss, mean loss per trial, maximum observed loss, and 95th-percentile VaR.

## Used in the wild

- **Lloyd's of London** — catastrophe risk models use Monte Carlo simulation to price insurance for events like hurricanes and pandemics
- **J.P. Morgan** — pioneered VaR as a firm-wide risk metric in the 1990s; now mandated by Basel III for all major banks
- **AWS/Azure SRE teams** — use similar simulation to set SLA thresholds and budget error rates across services

## Run it

```bash
cargo run -p risk-sampler
cargo run -p risk-sampler -- --trials 1000000 --seed 7
```

## Use it as a library

```rust
use risk_sampler::{RiskEvent, simulate};

let events = vec![
    RiskEvent { name: "outage".into(), probability: 0.1, max_loss: 10_000.0 },
];
let result = simulate(&events, 100_000, 42);
println!("VaR 95%: {:.2}", result.var_95);
```

## Rust concepts covered

- **Structs**: `RiskEvent` and `SimulationResult` as plain data containers
- **`rand` crate**: seeded RNG with `StdRng::seed_from_u64` for reproducibility
- **`clap` derive macro**: zero-boilerplate CLI argument parsing
- **`Vec` and sorting**: collecting trial results then sorting for percentile extraction
- **`f64` arithmetic**: accumulating floating-point sums with awareness of precision limits

## Builds on

Nothing — this is the foundation.

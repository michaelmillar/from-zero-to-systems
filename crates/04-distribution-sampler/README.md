# distribution-sampler

> Sample from Exponential, Poisson, and Weibull distributions — the building blocks of reliability engineering, load modelling, and queueing theory.

## ELI5

Different random processes produce different "shapes" of randomness. If buses arrive randomly, the time between buses follows an Exponential distribution — usually short, occasionally very long. The number of buses arriving per hour follows a Poisson distribution — usually close to the average, rarely way off. How long before a machine breaks down follows a Weibull distribution — which lets you model whether things tend to break early (like cheap electronics), randomly (like car accidents), or after long use (like engine wear). Each shape is a different tool for a different job.

## For the Educated Generalist

These three distributions are the core of **queueing theory** and **reliability engineering**.

**Exponential(λ)** is the unique continuous distribution with the *memoryless property*: the probability of an event in the next second is the same regardless of how long you've already waited. This makes it the natural model for request inter-arrival times in HTTP servers, radioactive decay, and hardware failure under constant stress. The parameter λ is the *rate* (events per unit time); the mean is 1/λ.

**Poisson(λ)** counts how many events occur in a fixed window when each event is independent and rare. It emerges naturally when you discretise an Exponential process: if inter-arrivals are Exponential(λ), then the count per unit time is Poisson(λ). This is why Poisson appears everywhere from network packet counts to insurance claim volumes to neuroscience spike trains.

**Weibull(k, λ)** is the workhorse of reliability engineering. Its *shape parameter k* determines the failure rate over time:
- **k < 1**: failure rate *decreases* — early defects dominate (infant mortality in electronics)
- **k = 1**: constant failure rate — equivalent to Exponential (random failures)
- **k > 1**: failure rate *increases* — wear-out dominates (bearing fatigue, material aging)

The Weibull's mean involves the Gamma function: `λ · Γ(1 + 1/k)`. We implement Gamma here via the Lanczos approximation — a glimpse into how special functions are computed numerically.

## What it does

Samples from Exponential, Poisson, and Weibull distributions using inverse-CDF and Knuth's algorithm. Prints observed mean, standard deviation, and P95 alongside theoretical values.

## Used in the wild

- **AWS/Google SRE** — Exponential and Poisson underpin queueing models (M/M/1) used to size server fleets and predict tail latency
- **Boeing / Airbus** — Weibull analysis is mandated by aviation regulators (FAA/EASA) for component lifetime certification
- **Netflix Chaos Engineering** — failure injection timings are drawn from Exponential distributions to simulate realistic hardware failure
- **CERN** — Poisson statistics govern particle collision rates; every particle physics measurement relies on it

## Run it

```bash
cargo run -p distribution-sampler
cargo run -p distribution-sampler -- -n 500000 --seed 7
```

## Use it as a library

```rust
use distribution_sampler::{Weibull, Sampler, sample_n};
use rand::SeedableRng;
use rand::rngs::StdRng;

let bearing = Weibull { shape: 2.5, scale: 10_000.0 }; // hours to failure
let mut rng = StdRng::seed_from_u64(42);
let lifetimes = sample_n(&bearing, 10_000, &mut rng);
println!("Mean lifetime: {:.0} hours", bearing.mean());
```

## Rust concepts covered

- **Traits with generics**: `Sampler` trait with `fn sample(&self, rng: &mut impl Rng)` — one interface, three implementations
- **Trait objects vs generics**: `sample_n` uses `impl Sampler` (static dispatch, zero overhead) — contrast with `Box<dyn Sampler>` for runtime polymorphism
- **Inverse CDF sampling**: deriving sample algorithms from the closed-form CDF — a fundamental numerical technique
- **`f64` special functions**: Lanczos Gamma approximation — how to implement mathematical functions without a maths library

## Builds on

- [`probability-engine`](../02-probability-engine/) — the `Distribution` trait pattern is extended here; Gamma sampling from crate 02 underpins the Weibull
- [`risk-sampler`](../01-risk-sampler/) — Exponential inter-arrival times directly extend the risk event model from crate 01

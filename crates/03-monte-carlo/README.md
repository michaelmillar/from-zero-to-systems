# monte-carlo

> Option pricing and portfolio Value at Risk via parallel Monte Carlo simulation.

## ELI5

Monte Carlo simulation is named after the casino in Monaco, because it uses random numbers the same way a casino does - run the game thousands of times and see what happens on average. To price an option (a bet on where a stock price will be in the future), we simulate thousands of possible futures for the stock, calculate the payout in each one, and average them out. The more futures we simulate, the more accurate our price. By spreading the work across all CPU cores at once, we can run millions of scenarios in seconds.

## For the Educated Generalist

An **option** is a financial contract giving the right to buy (call) or sell (put) an asset at a fixed price (*strike*) on a future date (*expiry*). Pricing it means answering: "what is the expected payoff, discounted to today?"

The elegant closed-form answer is **Black-Scholes**, derived by assuming asset prices follow geometric Brownian motion (GBM):

```
S_T = S_0 · exp((r - σ²/2)T + σ√T · Z), Z ~ N(0,1)
```

Monte Carlo validates this formula and extends beyond it - when you add jumps, stochastic volatility (Heston model), or path-dependent features (barrier options), there is no closed-form solution. Monte Carlo always works, at the cost of computation.

The trick for performance is **parallelism**: each simulated path is independent, so we shard the trial count across CPU cores using `rayon`. This is an embarrassingly parallel problem - the ideal case for `rayon`'s work-stealing thread pool.

**Value at Risk (VaR)** answers a different question: given a portfolio's historical or simulated return distribution, what loss will we not exceed in X% of periods? Regulators (Basel III, ESMA) require banks to compute and report VaR daily.

## What it does

Prices European call options via GBM simulation and compares against the Black-Scholes analytical solution. Also computes portfolio VaR at configurable confidence levels. Parallelised with `rayon` across all available CPU cores.

## Used in the wild

- **Goldman Sachs / JPMorgan** - Monte Carlo pricing desks run millions of paths per second for exotic derivatives (barrier options, Asian options, autocallables)
- **QuantLib** - the open-source C++ quant library uses MC extensively; this crate is a Rust reimagining of its core ideas
- **Basel III** - mandates VaR and Expected Shortfall reporting for all Tier 1 banks globally
- **OpenAI / DeepMind** - Monte Carlo Tree Search (MCTS) is the algorithm behind AlphaGo and game-playing AIs

## Run it

```bash
cargo run -p monte-carlo
cargo run -p monte-carlo -- --trials 2000000 --seed 7
```

## Use it as a library

```rust
use monte_carlo::{EuropeanCall, price_european_call, black_scholes_call};

let opt = EuropeanCall { spot: 100.0, strike: 105.0, rate: 0.05, volatility: 0.25, expiry: 0.5 };
let mc_price = price_european_call(&opt, 1_000_000, 42);
let bs_price = black_scholes_call(&opt);
println!("MC: {:.4} B-S: {:.4}", mc_price, bs_price);
```

## Rust concepts covered

- **`rayon` parallelism**: `into_par_iter()` distributes independent trials across CPU cores with zero data races - Rust's ownership rules guarantee this at compile time
- **Closures**: `.map(|chunk| { ... })` captures the seed and option parameters without heap allocation
- **`f64` precision**: GBM requires careful ordering of operations to avoid catastrophic cancellation
- **Seeded per-chunk RNG**: each parallel chunk gets `seed + chunk_id` - reproducible results without a shared mutable RNG
- **Abramowitz & Stegun**: hand-rolled normal CDF approximation to avoid pulling in a statistics crate

## Builds on

- [`probability-engine`](../02-probability-engine/) - `Distribution` trait and `Normal` distribution concepts carry forward
- [`risk-sampler`](../01-risk-sampler/) - VaR is the same percentile idea as in crate 01, now applied to return distributions

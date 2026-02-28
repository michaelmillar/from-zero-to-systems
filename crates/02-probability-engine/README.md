# probability-engine

> Probability distributions and Bayesian updating - the maths behind A/B tests, spam filters, and medical diagnostics.

## ELI5

Imagine you're trying to guess how often a website visitor clicks "Buy". You start with no idea - maybe it's 1%, maybe 90%. As more visitors come and some click, your guess gets sharper. Bayesian updating is a mathematical rule for changing your guess based on new evidence. It works like a scientist who starts uncertain and gets more confident with data, rather than a gambler who just counts heads.

## For the Educated Generalist

Bayesian inference treats probability as a degree of belief rather than a long-run frequency. We represent our uncertainty about an unknown parameter (like a conversion rate *p*) as a **prior distribution**. After observing data, we update it using Bayes' theorem to get a **posterior distribution**.

The clever part is choosing a **conjugate prior** - a prior distribution that, when updated with data from a given likelihood function, produces a posterior of the same family. For Bernoulli trials (yes/no events), the conjugate prior is the **Beta distribution**. The update rule is elegant: if you observe *s* successes and *f* failures, you simply add them to the prior's alpha and beta parameters. No integration required.

This is why the Beta-Bernoulli model is ubiquitous in industry: it's analytically exact, computationally trivial, and naturally expresses how confident you are in your estimate (wide Beta = uncertain, narrow Beta = confident).

## What it does

Provides a `Distribution` trait, `Bernoulli` and `Beta` distributions, and a `bayesian_update` function. The binary demonstrates Bayesian conversion rate estimation - starting from a uniform prior and updating it with observed visitor data.

## Used in the wild

- **Google Analytics** - uses Bayesian methods for conversion uplift estimates in experiment reports
- **Netflix** - Thompson sampling (Beta-Bernoulli bandit) for exploring recommendation strategies
- **MHRA / FDA** - adaptive Bayesian clinical trial designs, approved for drug and device testing
- **Naive Bayes spam filters** - still widely used in email systems (Gmail, SpamAssassin)

## Run it

```bash
cargo run -p probability-engine
```

## Use it as a library

```rust
use probability_engine::{Beta, Distribution, bayesian_update};

let prior = Beta { alpha: 1.0, beta: 1.0 }; // uniform - no prior knowledge
let posterior = bayesian_update(prior, 15, 85); // 15 successes, 85 failures
println!("Estimated rate: {:.2}%", posterior.mean() * 100.0);
```

## Rust concepts covered

- **Traits**: `Distribution` as a shared interface for sampling and moment calculation
- **Generics**: `fn sample(&self, rng: &mut impl Rng)` - accepts any RNG implementation
- **Structs**: plain data containers for distribution parameters (`alpha`, `beta`, `p`)
- **`f64` precision**: closed-form mean/variance vs sampling-based approximation
- **Iterators**: `(0..10).map(...).collect()` for batch sampling in the binary

## Builds on

- [`risk-sampler`](../01-risk-sampler/) - imported as a library; downstream crates can combine risk simulation with Bayesian probability estimation

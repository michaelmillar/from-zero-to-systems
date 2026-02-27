# statistics-core

> Descriptive statistics for sensor and telemetry data - mean, variance, percentiles, skewness, kurtosis, and IQR outlier detection.

## ELI5

Statistics is just ways of summarising a big pile of numbers into a few useful ones. The mean tells you the "typical" value. The variance tells you how spread out the numbers are. The median tells you what the middle value is (not affected by extreme outliers like the mean is). Percentiles tell you what value 95% of your data falls below. If a CPU temperature sensor is mostly reading 65°C but occasionally spikes to 100°C, statistics helps you see that the spike is an outlier and the true typical temperature is fine.

## For the Educated Generalist

Descriptive statistics splits into **measures of central tendency** (mean, median, mode) and **measures of dispersion** (variance, std dev, IQR). Choosing the right one matters:

- **Mean** is sensitive to outliers; a single extreme value pulls it significantly. Use it when data is roughly symmetric.
- **Median** is robust to outliers (a "resistant" statistic). Preferred for skewed distributions like house prices or income data.
- **Variance** uses squared deviations - which amplifies large errors but makes the maths tractable (it's the basis of the normal distribution's PDF). **Sample variance** divides by N-1 (Bessel's correction) to correct for the bias introduced by estimating the mean from the same sample.
- **Skewness** measures the asymmetry of the distribution. Positive skewness means a longer right tail (e.g. income distributions). **Kurtosis** measures "tailedness" - excess kurtosis > 0 (leptokurtic) means heavier tails than a normal distribution, common in financial returns and sensor spike events.
- **Z-scores** standardise data to zero mean and unit variance - essential before feeding data into any distance-based algorithm (k-means, PCA, neural nets).
- **IQR outlier detection** defines outliers as values beyond Q1 − 1.5·IQR or Q3 + 1.5·IQR. This is robust because IQR itself is resistant to outliers, unlike std-dev-based methods.

## What it does

Computes descriptive statistics over `f64` slices with proper error handling via `thiserror`. The binary simulates CPU temperature telemetry with injected thermal spikes and shows how skewness and kurtosis reveal the non-normality.

## Used in the wild

- **Datadog / Prometheus** - percentile computation (P50, P95, P99) for latency histograms is the core of SLA monitoring at every major cloud provider
- **pandas / NumPy** - the `.describe()` function is this crate in Python form; used by millions of data scientists daily
- **Bloomberg Terminal** - skewness and kurtosis of return distributions are standard risk metrics across fixed income and equities desks
- **CERN ROOT** - statistical functions over particle physics datasets with billions of measurements

## Run it

```bash
cargo run -p statistics-core
```

## Use it as a library

```rust
use statistics_core::{mean, percentile, iqr_outliers, summarise};

let data = vec![1.0, 2.0, 3.0, 100.0]; // 100.0 is an outlier
println!("Mean: {:.2}", mean(&data).unwrap());
println!("Median: {:.2}", statistics_core::median(&data).unwrap());
println!("P95: {:.2}", percentile(&data, 0.95).unwrap());
println!("Outliers: {:?}", iqr_outliers(&data).unwrap());
```

## Rust concepts covered

- **`thiserror`**: derive macro for ergonomic custom error types - the idiomatic Rust alternative to `anyhow` when you want typed errors
- **`Result<T, E>`**: every function that can fail returns `Result` - the compiler forces callers to handle errors
- **`?` operator**: propagates errors up the call stack without boilerplate
- **Struct with public fields**: `Summary` bundles all statistics into one return value rather than N separate function calls

## Builds on

- [`monte-carlo`](../03-monte-carlo/) - VaR in crate 03 is a specialised percentile; `statistics-core` generalises that to arbitrary distributions and metrics

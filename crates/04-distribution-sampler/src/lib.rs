// ============================================================
//  YOUR CHALLENGE - implement three probability distributions.
//
//  Exponential(lambda): models time between events.
//    Sample via inverse CDF: -ln(U) / lambda
//    Mean: 1 / lambda
//
//  Poisson(lambda): models event count in a fixed interval.
//    Sample via Knuth's algorithm: count exponential
//    inter-arrivals until cumulative product drops below e^(-lambda)
//    Mean: lambda
//
//  Weibull(shape k, scale lambda): generalises Exponential.
//    Sample via inverse CDF: lambda * (-ln(U))^(1/k)
//    Mean: lambda * Gamma(1 + 1/k)  [use gamma() below]
//
//  Hint: use rng.gen::<f64>().max(1e-15) to avoid ln(0).
//        gamma() is already implemented - use it for Weibull mean.
// ============================================================

use rand::Rng;

/// A shared interface for continuous and discrete distributions.
/// Mirrors the trait from probability-engine but lives here to avoid
/// pulling in the full crate for every distribution type.
pub trait Sampler {
    fn sample(&self, rng: &mut impl Rng) -> f64;
    /// Theoretical mean
    fn mean(&self) -> f64;
}

/// Exponential(lambda) - models time between events in a Poisson process.
/// Used for: request inter-arrival times, hardware failure intervals, call durations.
pub struct Exponential {
    pub lambda: f64, // rate parameter (events per unit time)
}

/// Poisson(lambda) - models the number of events in a fixed interval.
/// Used for: packet arrivals, server requests per second, defects per batch.
pub struct Poisson {
    pub lambda: f64, // expected events per interval
}

/// Weibull(shape k, scale lambda) - generalises Exponential; models component lifetimes.
/// k < 1: decreasing failure rate (infant mortality)
/// k = 1: constant failure rate (pure Exponential)
/// k > 1: increasing failure rate (wear-out)
pub struct Weibull {
    pub shape: f64,  // k
    pub scale: f64,  // lambda
}

impl Sampler for Exponential {
    fn sample(&self, rng: &mut impl Rng) -> f64 {
        todo!()
    }
    fn mean(&self) -> f64 { todo!() }
}

impl Sampler for Poisson {
    fn sample(&self, rng: &mut impl Rng) -> f64 {
        todo!()
    }
    fn mean(&self) -> f64 { todo!() }
}

impl Sampler for Weibull {
    fn sample(&self, rng: &mut impl Rng) -> f64 {
        todo!()
    }
    fn mean(&self) -> f64 {
        todo!()
    }
}

/// Lanczos approximation of the Gamma function (accurate to ~15 significant digits)
pub fn gamma(z: f64) -> f64 {
    const G: f64 = 7.0;
    const C: [f64; 9] = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    if z < 0.5 {
        std::f64::consts::PI / ((std::f64::consts::PI * z).sin() * gamma(1.0 - z))
    } else {
        let z = z - 1.0;
        let mut x = C[0];
        for (i, &c) in C[1..].iter().enumerate() {
            x += c / (z + i as f64 + 1.0);
        }
        let t = z + G + 0.5;
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(z + 0.5) * (-t).exp() * x
    }
}

/// Sample N values from any Sampler and return them as a Vec
pub fn sample_n(dist: &impl Sampler, n: usize, rng: &mut impl Rng) -> Vec<f64> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    const N: usize = 200_000;

    fn rng() -> StdRng { StdRng::seed_from_u64(42) }

    fn sample_mean(samples: &[f64]) -> f64 {
        samples.iter().sum::<f64>() / samples.len() as f64
    }

    #[test]
    fn exponential_sample_mean_within_2pct_of_theoretical() {
        let dist = Exponential { lambda: 2.0 };
        let samples = sample_n(&dist, N, &mut rng());
        let mean = sample_mean(&samples);
        assert!(
            (mean - dist.mean()).abs() / dist.mean() < 0.02,
            "mean {:.4} vs theoretical {:.4}", mean, dist.mean()
        );
    }

    #[test]
    fn exponential_samples_are_non_negative() {
        let dist = Exponential { lambda: 0.5 };
        let samples = sample_n(&dist, 10_000, &mut rng());
        assert!(samples.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn poisson_sample_mean_within_2pct_of_lambda() {
        let dist = Poisson { lambda: 5.0 };
        let samples = sample_n(&dist, N, &mut rng());
        let mean = sample_mean(&samples);
        assert!(
            (mean - dist.mean()).abs() / dist.mean() < 0.02,
            "mean {:.4} vs lambda {:.4}", mean, dist.mean()
        );
    }

    #[test]
    fn weibull_shape1_is_exponential() {
        // Weibull(k=1, lambda) = Exponential(1/lambda)
        let w = Weibull { shape: 1.0, scale: 2.0 };
        let e = Exponential { lambda: 0.5 };
        assert!((w.mean() - e.mean()).abs() < 1e-10);
    }

    #[test]
    fn weibull_mean_matches_gamma_formula() {
        let dist = Weibull { shape: 2.0, scale: 1.0 };
        // Gamma(1.5) = sqrt(pi)/2 approximately 0.8862
        let expected = gamma(1.5);
        assert!((dist.mean() - expected).abs() < 1e-6);
    }
}

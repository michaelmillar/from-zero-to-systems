// ============================================================
//  YOUR CHALLENGE - implement probability distributions.
//
//  Bernoulli(p): a single yes/no trial with probability p.
//  Beta(alpha, beta): conjugate prior for Bernoulli; models
//  uncertainty about p itself.
//
//  For Beta sampling, use sample_gamma (already implemented):
//    let x = sample_gamma(alpha, rng);
//    let y = sample_gamma(beta, rng);
//    Beta sample = x / (x + y)
//
//  Hint: use rng.gen::<f64>() for a uniform [0,1) sample.
// ============================================================

use rand::Rng;

pub trait Distribution {
    fn sample(&self, rng: &mut impl Rng) -> f64;
    fn mean(&self) -> f64;
    fn variance(&self) -> f64;
}

/// Bernoulli(p) - a single yes/no event with probability p
pub struct Bernoulli {
    pub p: f64,
}

/// Beta(alpha, beta) - the conjugate prior for Bernoulli; models uncertainty about p
pub struct Beta {
    pub alpha: f64,
    pub beta: f64,
}

/// Update a Beta prior given observed successes and failures (Bayesian update rule)
pub fn bayesian_update(prior: Beta, successes: u64, failures: u64) -> Beta {
    todo!()
}

impl Distribution for Bernoulli {
    fn sample(&self, rng: &mut impl Rng) -> f64 {
        todo!()
    }
    fn mean(&self) -> f64 { todo!() }
    fn variance(&self) -> f64 { todo!() }
}

impl Distribution for Beta {
    fn sample(&self, rng: &mut impl Rng) -> f64 {
        todo!()
    }
    fn mean(&self) -> f64 { todo!() }
    fn variance(&self) -> f64 { todo!() }
}

/// Marsaglia-Tsang method for Gamma(shape, 1) sampling. Requires shape >= 1.
fn sample_gamma(shape: f64, rng: &mut impl Rng) -> f64 {
    let shape = shape.max(1.0);
    let d = shape - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    loop {
        let u1: f64 = rng.gen::<f64>().max(1e-10);
        let u2: f64 = rng.gen::<f64>();
        let x = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let v = (1.0 + c * x).powi(3);
        if v > 0.0 {
            let u: f64 = rng.gen();
            if u < 1.0 - 0.0331 * (x * x) * (x * x) {
                return d * v;
            }
            if u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) {
                return d * v;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn bernoulli_mean_equals_p() {
        let b = Bernoulli { p: 0.3 };
        assert_eq!(b.mean(), 0.3);
    }

    #[test]
    fn bernoulli_variance_is_p_times_one_minus_p() {
        let b = Bernoulli { p: 0.4 };
        assert!((b.variance() - 0.24).abs() < 1e-10);
    }

    #[test]
    fn beta_mean_is_alpha_over_alpha_plus_beta() {
        let b = Beta { alpha: 2.0, beta: 5.0 };
        let expected = 2.0 / 7.0;
        assert!((b.mean() - expected).abs() < 1e-10);
    }

    #[test]
    fn bayesian_update_moves_mean_toward_data() {
        let prior = Beta { alpha: 1.0, beta: 1.0 };
        let prior_mean = prior.mean();
        let posterior = bayesian_update(prior, 8, 2);
        assert!(posterior.mean() > prior_mean);
        // posterior alpha=9, beta=3, mean = 9/12 = 0.75
        assert!((posterior.mean() - 9.0 / 12.0).abs() < 1e-10);
    }

    #[test]
    fn bernoulli_samples_are_zero_or_one() {
        let b = Bernoulli { p: 0.5 };
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..1000 {
            let s = b.sample(&mut rng);
            assert!(s == 0.0 || s == 1.0);
        }
    }

    #[test]
    fn beta_sample_is_in_unit_interval() {
        let b = Beta { alpha: 2.0, beta: 5.0 };
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..1000 {
            let s = b.sample(&mut rng);
            assert!((0.0..=1.0).contains(&s), "sample out of range: {}", s);
        }
    }
}

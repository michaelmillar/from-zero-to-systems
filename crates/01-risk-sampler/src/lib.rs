// ============================================================
//  YOUR CHALLENGE - simulate risk events and calculate
//  Value at Risk (VaR) using Monte Carlo simulation.
//
//  For each trial:
//    - for each event, roll a random number against its probability
//    - if it fires, sample a uniform loss in [0, max_loss]
//    - accumulate the trial total loss
//  After all trials, sort the losses and read off the 95th percentile.
//
//  Hint: seed a StdRng with seed_from_u64(seed) for reproducibility.
// ============================================================

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub struct RiskEvent {
    pub name: String,
    pub probability: f64,
    pub max_loss: f64,
}

pub struct SimulationResult {
    pub trials: u64,
    pub occurrences: u64,
    pub total_loss: f64,
    pub mean_loss_per_trial: f64,
    pub max_observed_loss: f64,
    pub var_95: f64,
}

pub fn simulate(events: &[RiskEvent], trials: u64, seed: u64) -> SimulationResult {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn certain_event_always_occurs() {
        let events = vec![RiskEvent {
            name: "always".into(),
            probability: 1.0,
            max_loss: 100.0,
        }];
        let result = simulate(&events, 1_000, 42);
        assert_eq!(result.occurrences, 1_000);
        assert!(result.total_loss > 0.0);
    }

    #[test]
    fn var_95_is_not_greater_than_max_possible_loss() {
        let events = vec![RiskEvent {
            name: "flood".into(),
            probability: 0.1,
            max_loss: 50_000.0,
        }];
        let result = simulate(&events, 100_000, 7);
        assert!(result.var_95 <= 50_000.0);
    }

    #[test]
    fn mean_loss_is_consistent_with_probability() {
        let prob = 0.2;
        let max_loss = 1000.0;
        let events = vec![RiskEvent {
            name: "outage".into(),
            probability: prob,
            max_loss,
        }];
        let result = simulate(&events, 500_000, 99);
        let expected = prob * max_loss / 2.0;
        let tolerance = expected * 0.05;
        assert!(
            (result.mean_loss_per_trial - expected).abs() < tolerance,
            "mean {:.2} not within 5% of expected {:.2}",
            result.mean_loss_per_trial,
            expected
        );
    }
}

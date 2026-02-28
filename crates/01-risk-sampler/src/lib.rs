// ============================================================
//  YOUR CHALLENGE - simulate risk events and calculate
//  Value at Risk (VaR) using Monte Carlo simulation.
//
//  WHAT YOU ARE BUILDING
//  ---------------------
//  A risk model used by insurance companies, banks, and trading
//  desks to answer: "In the worst 5% of years, how much do we lose?"
//  That figure is the 95th-percentile VaR.
//
//  THE ALGORITHM (plain English)
//  ------------------------------
//  Repeat `trials` times:
//    For each RiskEvent:
//      Roll a number between 0 and 1.
//      If that number < event.probability -> the event fires.
//      When it fires, pick a random loss between 0 and event.max_loss.
//    Record the total loss for this trial.
//
//  After all trials:
//    Sort the per-trial losses low -> high.
//    The value 95% of the way through that sorted list is your VaR 95%.
//
//  RESULT FIELDS
//  -------------
//  trials            — same as the input, echoed back
//  occurrences       — total number of times ANY event fired across ALL trials
//  total_loss        — sum of every individual event loss across ALL trials
//  mean_loss_per_trial — total_loss / trials  (average per run)
//  max_observed_loss — highest single-trial total loss seen
//  var_95            — the 95th-percentile trial loss (sorted_losses[0.95 * trials])
//
//  RUST SYNTAX NOTES
//  -----------------
//  Create the RNG:
//    let mut rng = StdRng::seed_from_u64(seed);
//
//  Generate a random f64 in [0.0, 1.0):
//    rng.gen::<f64>()      <- the ::<f64> "turbofish" picks the output type
//
//  Sort a Vec<f64> (f64 has no Ord, so you must use partial_cmp):
//    losses.sort_by(|a, b| a.partial_cmp(b).unwrap());
//
//  Maximum of a Vec<f64>:
//    losses.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
// ============================================================

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub struct RiskEvent {
    pub name: String,
    pub probability: f64,
    pub max_loss: f64,
}

pub struct SimulationResult {
    /// Same as the `trials` argument passed in.
    pub trials: u64,
    /// Total number of event firings across all trials and all events.
    pub occurrences: u64,
    /// Sum of all losses across all trials.
    pub total_loss: f64,
    /// total_loss / trials.
    pub mean_loss_per_trial: f64,
    /// Largest single-trial loss observed.
    pub max_observed_loss: f64,
    /// 95th-percentile trial loss (Value at Risk).
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

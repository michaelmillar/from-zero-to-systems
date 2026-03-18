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

    let occurrences: u64 = 0;
    let total_loss: u64 = 0 ;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut losses = Vec::new();
    let mut loss: f64 = 0.0;
    for trial in 0..trials {
    	for event in events {
    		if rng.gen::<f64>() < event.probability {
    			occurrences += 1;
    			loss = rng.gen::<f64>() * event.max_loss;
    			losses.append(loss);
    			total_loss += loss;
   
    		}
    	}
    }
    
    let sorted_losses = losses.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let max_observed_loss = losses.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let result = SimulationResult {
    trials: trials,
    	occurrences: occurrences,
    	total_loss: total_loss	,
    	 mean_loss_per_trial: total_loss / trials,
    	 max_observed_loss: max_observed_loss,
    	var_95:sorted_losses[0.95 * trials]
    		
    }	;
    result;
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

use probability_engine::{Beta, Distribution, bayesian_update};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);

    println!("=== Bayesian Conversion Rate Estimation ===\n");

    let prior = Beta { alpha: 1.0, beta: 1.0 };
    println!("Prior:     Beta({:.1}, {:.1})  mean = {:.4}", prior.alpha, prior.beta, prior.mean());

    // Observe 15 conversions from 100 visitors
    let posterior = bayesian_update(prior, 15, 85);
    println!("After 100 visitors, 15 converted:");
    println!("Posterior: Beta({:.1}, {:.1})  mean = {:.4}  variance = {:.6}",
        posterior.alpha, posterior.beta, posterior.mean(), posterior.variance());

    println!("\nPosterior samples (uncertainty about true conversion rate):");
    for i in 1..=10 {
        let s = posterior.sample(&mut rng);
        println!("  sample {:>2}: {:.4}", i, s);
    }
}

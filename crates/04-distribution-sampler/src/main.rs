use clap::Parser;
use distribution_sampler::{Exponential, Poisson, Sampler, Weibull, sample_n};
use rand::SeedableRng;
use rand::rngs::StdRng;

#[derive(Parser)]
#[command(name = "distribution-sampler", about = "Sample from Exponential, Poisson, and Weibull distributions")]
struct Args {
    #[arg(short = 'n', long, default_value_t = 50_000)]
    samples: usize,

    #[arg(short, long, default_value_t = 42)]
    seed: u64,
}

fn summarise(name: &str, samples: &[f64], theoretical_mean: f64) {
    let n = samples.len() as f64;
    let mean = samples.iter().sum::<f64>() / n;
    let var = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95 = sorted[(n * 0.95) as usize];
    println!("  {name}");
    println!("    Theoretical mean: {:>10.4}  Observed mean: {:>10.4}", theoretical_mean, mean);
    println!("    Std dev: {:>8.4}  P95: {:>10.4}", var.sqrt(), p95);
    println!();
}

fn main() {
    let args = Args::parse();
    let mut rng = StdRng::seed_from_u64(args.seed);

    println!("=== Distribution Sampler ({} samples) ===\n", args.samples);

    let exp = Exponential { lambda: 2.0 };
    let exp_samples = sample_n(&exp, args.samples, &mut rng);
    summarise("Exponential(λ=2)  — avg 0.5s between server requests", &exp_samples, exp.mean());

    let poi = Poisson { lambda: 10.0 };
    let poi_samples = sample_n(&poi, args.samples, &mut rng);
    summarise("Poisson(λ=10)     — packets arriving per millisecond", &poi_samples, poi.mean());

    let w_infant = Weibull { shape: 0.5, scale: 1000.0 };
    let w_samples = sample_n(&w_infant, args.samples, &mut rng);
    summarise("Weibull(k=0.5)    — infant mortality failure mode (early failures)", &w_samples, w_infant.mean());

    let w_wearout = Weibull { shape: 3.5, scale: 1000.0 };
    let w2_samples = sample_n(&w_wearout, args.samples, &mut rng);
    summarise("Weibull(k=3.5)    — wear-out failure mode (components age)", &w2_samples, w_wearout.mean());
}

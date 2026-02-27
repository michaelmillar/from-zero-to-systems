use clap::Parser;
use monte_carlo::{EuropeanCall, black_scholes_call, price_european_call, value_at_risk};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};

#[derive(Parser)]
#[command(name = "monte-carlo", about = "Option pricing and portfolio VaR via Monte Carlo")]
struct Args {
    #[arg(short, long, default_value_t = 500_000)]
    trials: u64,

    #[arg(short, long, default_value_t = 42)]
    seed: u64,
}

fn main() {
    let args = Args::parse();

    // --- European call option pricing ---
    println!("=== European Call Option Pricing ===");
    println!("  Spot=100, Strike=100, r=5%, σ=20%, T=1yr (at-the-money)\n");

    let atm = EuropeanCall { spot: 100.0, strike: 100.0, rate: 0.05, volatility: 0.20, expiry: 1.0 };
    let bs_price = black_scholes_call(&atm);
    let mc_price = price_european_call(&atm, args.trials, args.seed);

    println!("  Black-Scholes (analytical): {:>10.4}", bs_price);
    println!("  Monte Carlo ({:>7} trials): {:>10.4}", args.trials, mc_price);
    println!("  Error:                      {:>9.4}%\n", ((mc_price - bs_price) / bs_price).abs() * 100.0);

    // --- Portfolio VaR ---
    println!("=== Portfolio Value at Risk (1-day, 95% & 99%) ===");
    println!("  Simulating 10,000 daily returns (drift=0.05%, vol=1%)\n");

    let mut rng = StdRng::seed_from_u64(args.seed);
    let normal = Normal::new(0.0005, 0.01).unwrap();
    let returns: Vec<f64> = (0..10_000).map(|_| normal.sample(&mut rng)).collect();

    println!("  VaR 95%: {:>8.4}%  (lose more than this on 5% of days)",  value_at_risk(&returns, 0.95) * 100.0);
    println!("  VaR 99%: {:>8.4}%  (lose more than this on 1% of days)",  value_at_risk(&returns, 0.99) * 100.0);
}

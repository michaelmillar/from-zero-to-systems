use clap::Parser;
use risk_sampler::{RiskEvent, simulate};

#[derive(Parser)]
#[command(name = "risk-sampler", about = "Simulate risk event losses across a portfolio")]
struct Args {
    #[arg(short, long, default_value_t = 100_000)]
    trials: u64,

    #[arg(short, long, default_value_t = 42)]
    seed: u64,
}

fn main() {
    let args = Args::parse();

    let events = vec![
        RiskEvent { name: "Cyber attack".into(),       probability: 0.05, max_loss: 500_000.0  },
        RiskEvent { name: "Server outage".into(),      probability: 0.15, max_loss:  50_000.0  },
        RiskEvent { name: "Supply chain delay".into(), probability: 0.20, max_loss:  25_000.0  },
        RiskEvent { name: "Regulatory fine".into(),    probability: 0.02, max_loss: 1_000_000.0 },
    ];

    println!("Running {} trials...\n", args.trials);
    let result = simulate(&events, args.trials, args.seed);

    println!("=== Risk Simulation Results ===");
    println!("Trials:               {:>15}",       result.trials);
    println!("Event occurrences:    {:>15}",       result.occurrences);
    println!("Total loss:           {:>15.2}",     result.total_loss);
    println!("Mean loss / trial:    {:>15.2}",     result.mean_loss_per_trial);
    println!("Max observed loss:    {:>15.2}",     result.max_observed_loss);
    println!("VaR 95%:              {:>15.2}",     result.var_95);
}

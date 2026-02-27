use linear_regression::{fit, predict};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    let noise = Normal::new(0.0_f64, 50.0).unwrap();

    // Demand forecasting: units_sold ≈ 800 + 12*(temperature) - 5*(price_delta)
    println!("=== Demand Forecasting: units_sold ~ temperature + price ===\n");

    let n = 200;
    let x: Vec<Vec<f64>> = (0..n).map(|i| {
        let temp = 10.0 + (i as f64 % 30.0);
        let price_delta = -5.0 + (i as f64 % 10.0);
        vec![temp, price_delta]
    }).collect();

    let y: Vec<f64> = x.iter().map(|r| {
        800.0 + 12.0 * r[0] - 5.0 * r[1] + noise.sample(&mut rng)
    }).collect();

    let model = fit(&x, &y).expect("fit should succeed");

    println!("  Fitted model:");
    println!("    Intercept (β₀):          {:>10.2}  (true: 800)", model.coefficients[0]);
    println!("    Temperature coef (β₁):   {:>10.2}  (true: 12)", model.coefficients[1]);
    println!("    Price delta coef (β₂):   {:>10.2}  (true: -5)", model.coefficients[2]);
    println!("    R²:                       {:>10.4}", model.r_squared);

    println!("\n  Predictions:");
    let scenarios = [
        ("Hot day, no discount",  vec![35.0,  0.0]),
        ("Cold day, big discount", vec![12.0, -8.0]),
        ("Avg day, slight markup", vec![22.0,  3.0]),
    ];
    for (label, feats) in &scenarios {
        println!("    {:<28} → {:.0} units", label, predict(&model, feats));
    }
}

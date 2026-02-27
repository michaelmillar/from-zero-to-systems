use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};
use statistics_core::summarise;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);

    // Simulate CPU temperature sensor readings (°C) — slightly skewed by occasional spikes
    let normal = Normal::new(65.0_f64, 5.0).unwrap();
    let mut readings: Vec<f64> = (0..1000).map(|_| normal.sample(&mut rng)).collect();
    // Inject thermal spike outliers
    readings.extend_from_slice(&[98.5, 101.2, 99.8]);

    println!("=== CPU Temperature Telemetry ({} readings) ===\n", readings.len());

    let s = summarise(&readings).expect("non-empty data");
    println!("  Mean:       {:>8.2} °C", s.mean);
    println!("  Std dev:    {:>8.2} °C", s.std_dev);
    println!("  Median:     {:>8.2} °C", s.median);
    println!("  P5  / P95:  {:>8.2} °C  /  {:.2} °C", s.p5, s.p95);
    println!("  Skewness:   {:>8.4}  (>0 = right tail from spikes)", s.skewness);
    println!("  Ex. Kurt:   {:>8.4}  (>0 = heavier tail than normal)", s.kurtosis);
    println!("  Outliers:   {:>8}  (IQR method)", s.n_outliers);
}

use signal_processing::{analyse, rms, sine_wave};

fn main() {
    let sample_rate = 1000.0_f64; // 1 kHz
    let n = 2048;

    println!("=== Signal Processing: ECG-like Spectrum Analysis ===\n");

    // Simulate a simplified ECG: dominant heartbeat at 1.2 Hz (72 bpm)
    // + respiration artefact at 0.25 Hz + high-frequency noise component
    let heartbeat  = sine_wave(1.2,  sample_rate, n);
    let respiration = sine_wave(0.25, sample_rate, n);
    let hf_noise   = sine_wave(50.0, sample_rate, n); // 50 Hz mains interference

    let signal: Vec<f64> = heartbeat.iter()
        .zip(&respiration)
        .zip(&hf_noise)
        .map(|((h, r), n)| h + 0.3 * r + 0.1 * n)
        .collect();

    println!("  Signal components:");
    println!("    Heartbeat:          1.20 Hz  (72 bpm)    amplitude 1.0");
    println!("    Respiration:        0.25 Hz               amplitude 0.3");
    println!("    Mains interference: 50.0 Hz               amplitude 0.1");
    println!("  Sample rate: {} Hz,  N = {} samples\n", sample_rate, n);

    let result = analyse(&signal, sample_rate);
    println!("  Dominant frequency detected: {:.3} Hz", result.dominant_freq_hz);
    println!("  Signal RMS: {:.4}", rms(&signal));

    println!("\n  Top 5 frequency bins by magnitude:");
    let mut indexed: Vec<(usize, f64)> = result.magnitudes.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (bin, mag) in indexed.iter().take(5) {
        let freq = *bin as f64 * sample_rate / n as f64;
        println!("    bin {:>4} → {:>8.4} Hz   magnitude {:.6}", bin, freq, mag);
    }
}

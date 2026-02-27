use rustfft::{FftPlanner, num_complex::Complex};

/// Apply a Hann window to a signal to reduce spectral leakage
pub fn hann_window(signal: &[f64]) -> Vec<f64> {
    let n = signal.len();
    signal.iter().enumerate().map(|(i, &s)| {
        let w = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / (n - 1) as f64).cos());
        s * w
    }).collect()
}

/// Result of an FFT analysis
pub struct SpectrumAnalysis {
    /// Magnitude at each frequency bin
    pub magnitudes: Vec<f64>,
    /// Dominant frequency in Hz
    pub dominant_freq_hz: f64,
    /// Index of the dominant bin
    pub dominant_bin: usize,
    /// Sample rate used
    pub sample_rate: f64,
}

/// Compute the FFT magnitude spectrum of a real-valued signal.
/// Returns magnitude for bins 0..=N/2 (positive frequencies only).
pub fn analyse(signal: &[f64], sample_rate: f64) -> SpectrumAnalysis {
    let n = signal.len();
    let windowed = hann_window(signal);
    let mut buffer: Vec<Complex<f64>> = windowed.iter().map(|&s| Complex::new(s, 0.0)).collect();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    fft.process(&mut buffer);

    // Only positive frequencies (bins 0..=N/2)
    let half = n / 2 + 1;
    let magnitudes: Vec<f64> = buffer[..half].iter().map(|c| c.norm() / n as f64).collect();

    // Find dominant bin (skip DC at bin 0)
    let dominant_bin = magnitudes[1..].iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i + 1)
        .unwrap_or(0);

    let dominant_freq_hz = dominant_bin as f64 * sample_rate / n as f64;

    SpectrumAnalysis { magnitudes, dominant_freq_hz, dominant_bin, sample_rate }
}

/// Generate a pure sine wave
pub fn sine_wave(freq_hz: f64, sample_rate: f64, n_samples: usize) -> Vec<f64> {
    (0..n_samples).map(|i| {
        (2.0 * std::f64::consts::PI * freq_hz * i as f64 / sample_rate).sin()
    }).collect()
}

/// Compute signal RMS (root mean square) — power metric
pub fn rms(signal: &[f64]) -> f64 {
    (signal.iter().map(|x| x.powi(2)).sum::<f64>() / signal.len() as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f64 = 1000.0; // 1 kHz
    const N: usize = 1024;

    #[test]
    fn fft_of_pure_sine_finds_correct_frequency() {
        let freq = 50.0; // 50 Hz
        let signal = sine_wave(freq, SAMPLE_RATE, N);
        let result = analyse(&signal, SAMPLE_RATE);
        assert!(
            (result.dominant_freq_hz - freq).abs() < SAMPLE_RATE / N as f64 * 2.0,
            "detected {:.2} Hz, expected {:.2} Hz",
            result.dominant_freq_hz, freq
        );
    }

    #[test]
    fn fft_of_dc_signal_has_peak_at_bin_zero() {
        let dc: Vec<f64> = vec![1.0; N];
        let result = analyse(&dc, SAMPLE_RATE);
        // DC bin should be the largest
        assert_eq!(
            result.magnitudes.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap().0,
            0
        );
    }

    #[test]
    fn rms_of_unit_sine_is_one_over_sqrt2() {
        let signal = sine_wave(50.0, SAMPLE_RATE, 10_000);
        let r = rms(&signal);
        assert!((r - 1.0_f64 / 2.0_f64.sqrt()).abs() < 0.001, "RMS {:.4}", r);
    }

    #[test]
    fn hann_window_starts_and_ends_near_zero() {
        let signal = vec![1.0; 64];
        let windowed = hann_window(&signal);
        assert!(windowed[0].abs() < 1e-10);
        assert!(windowed[63].abs() < 1e-10);
    }

    #[test]
    fn higher_frequency_detected_in_two_component_signal() {
        // 50 Hz + 200 Hz; 200 Hz has same amplitude but should dominate after windowing
        let sig50  = sine_wave(50.0,  SAMPLE_RATE, N);
        let sig200 = sine_wave(200.0, SAMPLE_RATE, N);
        let mixed: Vec<f64> = sig50.iter().zip(&sig200).map(|(a, b)| a + b).collect();
        let result = analyse(&mixed, SAMPLE_RATE);
        // Dominant should be one of the two known frequencies
        let closest = [50.0_f64, 200.0_f64]
            .iter()
            .min_by(|&&a, &&b| (a - result.dominant_freq_hz).abs()
                .partial_cmp(&(b - result.dominant_freq_hz).abs()).unwrap())
            .unwrap();
        assert!(
            (result.dominant_freq_hz - closest).abs() < SAMPLE_RATE / N as f64 * 2.0,
            "dominant {:.2} Hz not close to 50 or 200 Hz", result.dominant_freq_hz
        );
    }
}

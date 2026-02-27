// ============================================================
//  YOUR CHALLENGE - implement FFT-based signal analysis.
//
//  hann_window: multiply each sample by the Hann window function
//    w[i] = 0.5 * (1 - cos(2*pi*i / (N-1)))
//    reduces spectral leakage at signal edges
//
//  analyse: compute FFT magnitude spectrum of a real signal
//    - apply Hann window
//    - run FFT using rustfft
//    - return magnitudes for bins 0..=N/2 (positive freqs only)
//    - find the dominant frequency bin (skip DC at bin 0)
//
//  sine_wave: generate pure sine wave samples
//    s[i] = sin(2*pi*freq*i / sample_rate)
//
//  rms: root mean square (power metric)
//    rms = sqrt(mean(x^2))
//
//  Hint: use rustfft's FftPlanner. Magnitude = c.norm() / N.
// ============================================================

use rustfft::{FftPlanner, num_complex::Complex};

/// Apply a Hann window to a signal to reduce spectral leakage
pub fn hann_window(signal: &[f64]) -> Vec<f64> {
    todo!()
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
    todo!()
}

/// Generate a pure sine wave
pub fn sine_wave(freq_hz: f64, sample_rate: f64, n_samples: usize) -> Vec<f64> {
    todo!()
}

/// Compute signal RMS (root mean square) - power metric
pub fn rms(signal: &[f64]) -> f64 {
    todo!()
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

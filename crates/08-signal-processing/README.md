# signal-processing

> FFT-based signal analysis — ECG heart rate detection, audio DSP, vibration monitoring, and RF spectrum analysis.

## ELI5

Sounds, heartbeats, and vibrations are all just waves that repeat at different speeds. A guitar string vibrates at 440 times per second to make the note A. The Fourier transform is a mathematical trick that takes any complicated wave and tells you exactly which simple repeating waves are hidden inside it — like separating a chord into its individual notes. We use this for everything from noise-cancelling headphones to medical heart monitors to the Wi-Fi signal reaching your phone.

## For the Educated Generalist

The **Discrete Fourier Transform (DFT)** decomposes a signal of N samples into N complex-valued frequency components. The naïve DFT is O(N²) — too slow for real use. The **Fast Fourier Transform (FFT)**, specifically the **Cooley-Tukey radix-2 algorithm**, reduces this to O(N log N) by recursively splitting the DFT into smaller DFTs of even and odd-indexed elements. This is one of the most important algorithms in all of computing.

Two practical considerations:

1. **Spectral leakage**: if the signal's frequency isn't exactly an integer multiple of the bin spacing (fs/N), energy "leaks" from the true bin into neighbouring bins. The **Hann window** (applied before the FFT) tapers the signal to zero at both ends, dramatically reducing leakage at the cost of slightly wider peaks. This is why windowing is standard practice in audio and RF analysis.

2. **Bin resolution**: the frequency resolution is Δf = fs/N. With sample rate 1000 Hz and N=2048, each bin is ~0.49 Hz wide. A heartbeat at 1.2 Hz sits between bins 2 (0.98 Hz) and 3 (1.46 Hz) — both will show energy. To resolve closer frequencies, increase N (more samples) or increase fs.

**RMS** (root mean square) is the square root of the mean of squared values. For a sine wave, RMS = amplitude/√2. It's the correct measure of signal power — equivalent to the DC voltage that would deliver the same power to a resistor.

## What it does

Applies a Hann window, runs the FFT via `rustfft`, identifies the dominant frequency, and reports the top magnitude bins. The binary simulates an ECG-like signal with a 1.2 Hz heartbeat, respiratory artefact, and 50 Hz mains interference.

## Used in the wild

- **Philips Healthcare / GE Medical** — ECG and EEG monitors compute real-time FFTs to detect arrhythmias, seizures, and sleep stages
- **Shazam** — identifies songs by computing FFT spectrograms of short audio clips and matching their frequency fingerprints
- **LIGO** — detected gravitational waves from black hole mergers by running FFT analysis over laser interferometer data
- **Wi-Fi / 5G chipsets** — OFDM (Orthogonal Frequency Division Multiplexing) uses the FFT to encode and decode data across hundreds of parallel frequency carriers

## Run it

```bash
cargo run -p signal-processing
```

## Use it as a library

```rust
use signal_processing::{sine_wave, analyse, rms};

let sample_rate = 44_100.0; // CD quality
let signal = sine_wave(440.0, sample_rate, 4096); // Concert A
let spectrum = analyse(&signal, sample_rate);
println!("Dominant: {:.1} Hz", spectrum.dominant_freq_hz);
println!("RMS: {:.4}", rms(&signal));
```

## Rust concepts covered

- **External crate (`rustfft`)**: using a high-performance, well-tested library rather than reimplementing Cooley-Tukey from scratch — the right engineering trade-off
- **Complex numbers**: `Complex<f64>` from `num-complex`; understanding that FFT output is inherently complex even for real inputs
- **Iterator chaining**: composing `zip`, `map`, `enumerate`, `max_by` to process signal data without intermediate allocation
- **`f64::consts::PI`**: using the standard library's constants rather than magic numbers

## Builds on

- [`matrix-math`](../06-matrix-math/) — the DFT can be expressed as a matrix-vector product `X = Wₙx` where Wₙ is the DFT matrix; this crate replaces that O(N²) multiply with the O(N log N) FFT algorithm

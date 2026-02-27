// ============================================================
//  YOUR CHALLENGE — implement a feedforward neural network
//  with backpropagation from scratch.
//
//  Architecture: [input_size, ...hidden_sizes..., output_size]
//  Activation:   sigmoid (all layers)
//  Loss:         binary cross-entropy (BCE) for classification
//
//  Forward pass:
//    z^l = W^l · a^{l-1} + b^l
//    a^l = sigmoid(z^l)
//
//  Backprop (BCE + sigmoid simplifies beautifully):
//    δ^L = a^L - y                     (output layer)
//    δ^l = (W^{l+1})ᵀ · δ^{l+1} ⊙ sigmoid'(z^l)
//    ΔW^l = δ^l ⊗ (a^{l-1})ᵀ
//    Δb^l = δ^l
//
//  Used in: image classifiers, fraud detection, recommendation.
// ============================================================

use rand::Rng;

pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

pub fn sigmoid_deriv(x: f64) -> f64 {
    let s = sigmoid(x);
    s * (1.0 - s)
}

pub fn relu(x: f64) -> f64 { x.max(0.0) }

pub fn relu_deriv(x: f64) -> f64 { if x > 0.0 { 1.0 } else { 0.0 } }

pub struct Network {
    // weights[l] shape: (n[l+1], n[l]) — weights[l][j][k] = weight from neuron k in layer l to neuron j in layer l+1
    pub weights: Vec<Vec<Vec<f64>>>,
    pub biases:  Vec<Vec<f64>>,
    pub sizes: Vec<usize>,
}

impl Network {
    /// Build a network with Xavier initialisation.
    pub fn new_with_rng(sizes: &[usize], rng: &mut impl Rng) -> Self {
        assert!(sizes.len() >= 2, "need at least input + output layer");
        let mut weights = Vec::new();
        let mut biases  = Vec::new();
        for l in 0..sizes.len() - 1 {
            let fan_in  = sizes[l];
            let fan_out = sizes[l + 1];
            let scale = (6.0_f64 / (fan_in + fan_out) as f64).sqrt();
            let w: Vec<Vec<f64>> = (0..fan_out)
                .map(|_| (0..fan_in).map(|_| rng.gen_range(-scale..scale)).collect())
                .collect();
            weights.push(w);
            biases.push(vec![0.0; fan_out]);
        }
        Self { weights, biases, sizes: sizes.to_vec() }
    }

    /// Run a single forward pass, returning the output activations.
    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        let mut a = input.to_vec();
        for (w, b) in self.weights.iter().zip(self.biases.iter()) {
            a = w.iter().zip(b.iter())
                .map(|(row, bias)| sigmoid(row.iter().zip(a.iter()).map(|(wi, ai)| wi * ai).sum::<f64>() + bias))
                .collect();
        }
        a
    }

    /// Train on a dataset for `epochs` passes, using online gradient descent.
    pub fn train(&mut self, inputs: &[Vec<f64>], targets: &[Vec<f64>], epochs: usize, lr: f64) {
        for _ in 0..epochs {
            for (x, y) in inputs.iter().zip(targets.iter()) {
                self.backprop(x, y, lr);
            }
        }
    }

    fn backprop(&mut self, input: &[f64], target: &[f64], lr: f64) {
        let n_layers = self.weights.len();

        // ── forward pass, storing z and a at every layer ──────────────────
        let mut activations: Vec<Vec<f64>> = vec![input.to_vec()];
        let mut pre_acts:    Vec<Vec<f64>> = Vec::new();
        let mut a = input.to_vec();
        for (w, b) in self.weights.iter().zip(self.biases.iter()) {
            let z: Vec<f64> = w.iter().zip(b.iter())
                .map(|(row, bias)| row.iter().zip(a.iter()).map(|(wi, ai)| wi * ai).sum::<f64>() + bias)
                .collect();
            a = z.iter().map(|&zi| sigmoid(zi)).collect();
            pre_acts.push(z);
            activations.push(a.clone());
        }

        // ── output layer delta: BCE + sigmoid → δ = a - y (no sigmoid_deriv) ──
        let out = &activations[n_layers];
        let mut delta: Vec<f64> = out.iter().zip(target.iter()).map(|(a, y)| a - y).collect();

        // ── backwards through layers ───────────────────────────────────────
        for l in (0..n_layers).rev() {
            let a_prev = activations[l].clone();

            for j in 0..self.weights[l].len() {
                for k in 0..self.weights[l][j].len() {
                    self.weights[l][j][k] -= lr * delta[j] * a_prev[k];
                }
                self.biases[l][j] -= lr * delta[j];
            }

            // Propagate delta to the layer below (not needed for l=0)
            if l > 0 {
                let prev_size = self.weights[l][0].len();
                let mut new_delta = vec![0.0; prev_size];
                for k in 0..prev_size {
                    new_delta[k] = self.weights[l].iter().zip(delta.iter())
                        .map(|(row, d)| row[k] * d)
                        .sum::<f64>()
                        * sigmoid_deriv(pre_acts[l - 1][k]);
                }
                delta = new_delta;
            }
        }
    }

    /// Fraction of samples predicted correctly (threshold = 0.5 per output).
    pub fn accuracy(&self, inputs: &[Vec<f64>], targets: &[Vec<f64>]) -> f64 {
        let correct = inputs.iter().zip(targets.iter())
            .filter(|(x, y)| {
                let out  = self.forward(x);
                let pred: Vec<bool> = out.iter().map(|&a| a > 0.5).collect();
                let exp:  Vec<bool> = y.iter().map(|&t| t > 0.5).collect();
                pred == exp
            })
            .count();
        correct as f64 / inputs.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    mod activations {
        use super::*;

        #[test]
        fn sigmoid_of_zero_is_half() {
            assert!((sigmoid(0.0) - 0.5).abs() < 1e-10);
        }

        #[test]
        fn sigmoid_saturates_to_bounds() {
            assert!(sigmoid(100.0) > 0.999);
            assert!(sigmoid(-100.0) < 0.001);
        }

        #[test]
        fn relu_of_negative_is_zero() {
            assert_eq!(relu(-5.0), 0.0);
            assert_eq!(relu(-0.001), 0.0);
        }

        #[test]
        fn relu_of_positive_is_unchanged() {
            assert_eq!(relu(3.7), 3.7);
            assert_eq!(relu(0.0), 0.0);
        }
    }

    mod network {
        use super::*;

        #[test]
        fn forward_output_shape_is_correct() {
            let mut rng = StdRng::seed_from_u64(0);
            let net = Network::new_with_rng(&[3, 5, 2], &mut rng);
            let out = net.forward(&[0.1, 0.2, 0.3]);
            assert_eq!(out.len(), 2);
        }

        #[test]
        fn untrained_output_is_in_valid_range() {
            let mut rng = StdRng::seed_from_u64(1);
            let net = Network::new_with_rng(&[2, 4, 1], &mut rng);
            let out = net.forward(&[0.5, 0.5]);
            assert!(out[0] > 0.0 && out[0] < 1.0, "sigmoid output must be in (0,1)");
        }

        #[test]
        fn learns_xor() {
            // XOR: (0,0)→0, (0,1)→1, (1,0)→1, (1,1)→0
            let inputs  = vec![vec![0.0,0.0], vec![0.0,1.0], vec![1.0,0.0], vec![1.0,1.0]];
            let targets = vec![vec![0.0],     vec![1.0],     vec![1.0],     vec![0.0]];

            let mut rng = StdRng::seed_from_u64(42);
            let mut net = Network::new_with_rng(&[2, 8, 1], &mut rng);
            net.train(&inputs, &targets, 20_000, 0.5);

            let acc = net.accuracy(&inputs, &targets);
            assert!(acc >= 0.75,
                "XOR network should get at least 3/4 examples correct, got {acc}");
        }

        #[test]
        fn accuracy_improves_with_training() {
            let inputs  = vec![vec![0.0,0.0], vec![0.0,1.0], vec![1.0,0.0], vec![1.0,1.0]];
            let targets = vec![vec![0.0],     vec![1.0],     vec![1.0],     vec![0.0]];

            let mut rng = StdRng::seed_from_u64(99);
            let mut net = Network::new_with_rng(&[2, 4, 1], &mut rng);

            // Collect loss before and after training
            let loss_before: f64 = inputs.iter().zip(targets.iter())
                .map(|(x, y)| { let o = net.forward(x); (o[0] - y[0]).powi(2) })
                .sum();

            net.train(&inputs, &targets, 5_000, 0.5);

            let loss_after: f64 = inputs.iter().zip(targets.iter())
                .map(|(x, y)| { let o = net.forward(x); (o[0] - y[0]).powi(2) })
                .sum();

            assert!(loss_after < loss_before,
                "loss should decrease after training: before={loss_before:.4}, after={loss_after:.4}");
        }
    }
}

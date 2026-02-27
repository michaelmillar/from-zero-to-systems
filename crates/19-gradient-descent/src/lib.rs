// ============================================================
//  YOUR CHALLENGE — implement SGD and Adam optimisers.
//
//  Gradient descent is the engine behind all of ML.
//  Given a loss function L(θ) and its gradient ∇L(θ),
//  each step moves θ in the direction that decreases L.
//
//  SGD with momentum:
//    v  ← β·v + (1-β)·g           (velocity)
//    θ  ← θ - lr·v
//
//  Adam (Adaptive Moment Estimation):
//    m  ← β₁·m + (1-β₁)·g        (first moment)
//    v  ← β₂·v + (1-β₂)·g²       (second moment)
//    m̂  = m / (1 - β₁ᵗ)          (bias correction)
//    v̂  = v / (1 - β₂ᵗ)
//    θ  ← θ - lr · m̂ / (√v̂ + ε)
//
//  Used in: PyTorch, TensorFlow, every neural network trainer.
// ============================================================

/// Stochastic Gradient Descent with momentum.
pub struct Sgd {
    pub lr: f64,
    pub momentum: f64,
    velocity: Vec<f64>,
}

impl Sgd {
    pub fn new(lr: f64, momentum: f64) -> Self {
        Self { lr, momentum, velocity: Vec::new() }
    }

    /// Move `params` one step in the negative gradient direction.
    pub fn step(&mut self, params: &mut Vec<f64>, grads: &[f64]) {
        if self.velocity.len() != params.len() {
            self.velocity = vec![0.0; params.len()];
        }
        for i in 0..params.len() {
            self.velocity[i] = self.momentum * self.velocity[i]
                + (1.0 - self.momentum) * grads[i];
            params[i] -= self.lr * self.velocity[i];
        }
    }
}

/// Adam — Adaptive Moment Estimation.
pub struct Adam {
    pub lr: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub epsilon: f64,
    pub m: Vec<f64>,  // first moment
    pub v: Vec<f64>,  // second moment
    pub t: usize,     // step count (for bias correction)
}

impl Adam {
    pub fn new(lr: f64, beta1: f64, beta2: f64, epsilon: f64) -> Self {
        Self { lr, beta1, beta2, epsilon, m: Vec::new(), v: Vec::new(), t: 0 }
    }

    pub fn step(&mut self, params: &mut Vec<f64>, grads: &[f64]) {
        if self.m.len() != params.len() {
            self.m = vec![0.0; params.len()];
            self.v = vec![0.0; params.len()];
        }
        self.t += 1;
        let bc1 = 1.0 - self.beta1.powi(self.t as i32);
        let bc2 = 1.0 - self.beta2.powi(self.t as i32);
        for i in 0..params.len() {
            self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * grads[i];
            self.v[i] = self.beta2 * self.v[i] + (1.0 - self.beta2) * grads[i].powi(2);
            let m_hat = self.m[i] / bc1;
            let v_hat = self.v[i] / bc2;
            params[i] -= self.lr * m_hat / (v_hat.sqrt() + self.epsilon);
        }
    }
}

/// Estimate the gradient of f at x via central finite differences.
pub fn numerical_gradient<F: Fn(&[f64]) -> f64>(f: &F, x: &[f64], h: f64) -> Vec<f64> {
    let mut grad = vec![0.0; x.len()];
    let mut x_fwd = x.to_vec();
    let mut x_bwd = x.to_vec();
    for i in 0..x.len() {
        x_fwd[i] = x[i] + h;
        x_bwd[i] = x[i] - h;
        grad[i] = (f(&x_fwd) - f(&x_bwd)) / (2.0 * h);
        x_fwd[i] = x[i];
        x_bwd[i] = x[i];
    }
    grad
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sgd_tests {
        use super::*;

        #[test]
        fn zero_lr_produces_no_update() {
            let mut opt = Sgd::new(0.0, 0.0);
            let mut params = vec![5.0, -3.0];
            opt.step(&mut params, &[1.0, -1.0]);
            assert_eq!(params, vec![5.0, -3.0]);
        }

        #[test]
        fn step_moves_in_negative_gradient_direction() {
            let mut opt = Sgd::new(0.1, 0.0);
            let mut params = vec![2.0];
            opt.step(&mut params, &[4.0]); // positive grad → decrease
            assert!(params[0] < 2.0, "should move left when grad is positive");
        }

        #[test]
        fn converges_to_minimum_of_x_squared() {
            // f(x) = x²  →  grad = 2x  →  min at x = 0
            let mut opt = Sgd::new(0.1, 0.0);
            let mut x = vec![10.0_f64];
            for _ in 0..200 {
                let g = vec![2.0 * x[0]];
                opt.step(&mut x, &g);
            }
            assert!(x[0].abs() < 0.1, "should converge near 0, got {}", x[0]);
        }

        #[test]
        fn momentum_accumulates_velocity() {
            let mut opt = Sgd::new(0.01, 0.9);
            let mut params = vec![0.0];
            opt.step(&mut params, &[1.0]);
            let step1 = params[0].abs();
            opt.step(&mut params, &[1.0]);
            let step2 = (params[0].abs() - step1).abs();
            assert!(step2 > step1 * 0.9,
                "momentum should increase effective step size: {step1} → {step2}");
        }
    }

    mod adam_tests {
        use super::*;

        #[test]
        fn step_moves_in_negative_gradient_direction() {
            let mut opt = Adam::new(0.01, 0.9, 0.999, 1e-8);
            let mut params = vec![3.0];
            opt.step(&mut params, &[6.0]);
            assert!(params[0] < 3.0, "Adam should move in negative gradient direction");
        }

        #[test]
        fn moments_update_after_step() {
            let mut opt = Adam::new(0.01, 0.9, 0.999, 1e-8);
            let mut params = vec![1.0];
            opt.step(&mut params, &[2.0]);
            assert!(opt.m[0] > 0.0, "first moment should be positive");
            assert!(opt.v[0] > 0.0, "second moment should be positive");
            assert_eq!(opt.t, 1);
        }

        #[test]
        fn converges_to_minimum_of_quadratic() {
            // f(x,y) = x² + y²  →  min at (0, 0)
            let mut opt = Adam::new(0.1, 0.9, 0.999, 1e-8);
            let mut params = vec![5.0, -4.0];
            for _ in 0..500 {
                let g = vec![2.0 * params[0], 2.0 * params[1]];
                opt.step(&mut params, &g);
            }
            assert!(params[0].abs() < 0.01, "x should reach 0, got {}", params[0]);
            assert!(params[1].abs() < 0.01, "y should reach 0, got {}", params[1]);
        }
    }

    mod helpers {
        use super::*;

        #[test]
        fn numerical_gradient_of_x_squared_is_2x() {
            let f = |x: &[f64]| x[0] * x[0];
            let g = numerical_gradient(&f, &[3.0], 1e-5);
            assert!((g[0] - 6.0).abs() < 1e-4, "expected ≈6.0, got {}", g[0]);
        }

        #[test]
        fn numerical_gradient_of_constant_is_zero() {
            let f = |_: &[f64]| 42.0;
            let g = numerical_gradient(&f, &[1.0, 2.0, 3.0], 1e-5);
            for &gi in &g {
                assert!(gi.abs() < 1e-6, "gradient of constant should be 0, got {gi}");
            }
        }
    }
}

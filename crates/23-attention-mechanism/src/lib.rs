// ============================================================
//  YOUR CHALLENGE - implement scaled dot-product attention.
//
//  This is the computational core of every Transformer model -
//  GPT, BERT, LLaMA, ViT, Whisper, Claude.
//
//  Attention(Q, K, V) = softmax(Q K^T / sqrt(d_k)) * V
//
//  Where:
//    Q - queries:  [seq_len x d_k]
//    K - keys:     [seq_len x d_k]
//    V - values:   [seq_len x d_v]
//    d_k - key dimension (used for scaling to prevent vanishing gradients)
//
//  Causal masking: set future positions to -infinity before softmax
//  so position i can only attend to positions 0..i.
//
//  Depends on: matrix-math (crate 06) for matrix multiplication.
// ============================================================

use matrix_math::Matrix;

/// Row-wise softmax: each row sums to 1.
/// Subtracts max for numerical stability.
pub fn softmax(x: &[f64]) -> Vec<f64> {
    todo!()
}

/// Scaled dot-product attention.
///
/// - `q`, `k`: `[seq_len x d_k]`
/// - `v`:      `[seq_len x d_v]`
/// - `causal_mask`: if true, upper-triangular positions are masked to -infinity
///
/// Returns output of shape `[seq_len x d_v]`.
pub fn scaled_dot_product_attention(
    q: &[Vec<f64>],
    k: &[Vec<f64>],
    v: &[Vec<f64>],
    causal_mask: bool,
) -> Vec<Vec<f64>> {
    todo!()
}

// -- matrix helpers using crate 06 ─────────────────────────────────────────

fn to_matrix(rows: &[Vec<f64>]) -> Matrix {
    let r = rows.len();
    let c = if r > 0 { rows[0].len() } else { 0 };
    Matrix::from_vec(r, c, rows.iter().flatten().copied().collect())
}

fn from_matrix(m: &Matrix, rows: usize, cols: usize) -> Vec<Vec<f64>> {
    (0..rows).map(|i| (0..cols).map(|j| m[(i, j)]).collect()).collect()
}

fn matmul_2d(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let (rows, cols) = (a.len(), b[0].len());
    let result = to_matrix(a).matmul(&to_matrix(b)).expect("matmul dimension mismatch");
    from_matrix(&result, rows, cols)
}

fn transpose_2d(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let (rows, cols) = (m.len(), m[0].len());
    let result = to_matrix(m).transpose();
    from_matrix(&result, cols, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod softmax_tests {
        use super::*;

        #[test]
        fn outputs_sum_to_one() {
            let x = vec![1.0, 2.0, 3.0, 4.0];
            let s: f64 = softmax(&x).iter().sum();
            assert!((s - 1.0).abs() < 1e-10, "softmax sum = {s}");
        }

        #[test]
        fn uniform_input_produces_uniform_output() {
            let x = vec![1.0, 1.0, 1.0, 1.0];
            let s = softmax(&x);
            for &si in &s {
                assert!((si - 0.25).abs() < 1e-10, "expected 0.25, got {si}");
            }
        }

        #[test]
        fn large_value_dominates() {
            let x = vec![0.0, 0.0, 100.0, 0.0];
            let s = softmax(&x);
            assert!(s[2] > 0.999, "large logit should dominate: s[2]={}", s[2]);
        }
    }

    mod attention_tests {
        use super::*;

        #[test]
        fn output_shape_matches_value_shape() {
            let q = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
            let k = q.clone();
            let v = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
            let out = scaled_dot_product_attention(&q, &k, &v, false);
            assert_eq!(out.len(), 2,    "seq_len should match");
            assert_eq!(out[0].len(), 3, "d_v should match value dim");
        }

        #[test]
        fn identical_q_k_gives_uniform_attention() {
            // When all query rows are the same and all key rows are the same,
            // attention weights across positions should be equal.
            let q = vec![vec![1.0, 0.0]; 4];
            let k = vec![vec![1.0, 0.0]; 4];
            let v = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0]];
            let out = scaled_dot_product_attention(&q, &k, &v, false);
            // Each output row should be the average of V: 2.5
            for row in &out {
                assert!((row[0] - 2.5).abs() < 1e-6, "expected 2.5, got {}", row[0]);
            }
        }

        #[test]
        fn causal_mask_prevents_future_attention() {
            // Position 0 with causal mask: can only see itself.
            // So output[0] should equal v[0].
            let q = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]];
            let k = q.clone();
            let v = vec![vec![10.0], vec![20.0], vec![30.0]];
            let out = scaled_dot_product_attention(&q, &k, &v, true);
            assert!((out[0][0] - 10.0).abs() < 1e-5,
                "position 0 should attend only to itself, expected 10.0, got {}", out[0][0]);
        }

        #[test]
        fn attention_weights_are_non_negative() {
            let q = vec![vec![0.5, -0.3], vec![-0.1, 0.8]];
            let k = vec![vec![0.2,  0.6], vec![ 0.9, 0.1]];
            let v = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
            let out = scaled_dot_product_attention(&q, &k, &v, false);
            for row in &out {
                for &x in row {
                    assert!(x >= -1e-10, "attention output should be non-negative: {x}");
                }
            }
        }
    }
}

use matrix_math::Matrix;
use statistics_core::{mean, StatsError};

#[derive(Debug)]
pub struct LinearModel {
    /// Fitted coefficients: beta[0] = intercept, beta[1..] = feature weights
    pub coefficients: Vec<f64>,
    pub r_squared: f64,
    pub residuals: Vec<f64>,
}

/// Fit OLS via normal equations: β = (XᵀX)⁻¹Xᵀy
/// `x` is a 2D slice in row-major order (n_samples × n_features, no bias column).
/// A bias column of 1s is prepended automatically.
pub fn fit(x: &[Vec<f64>], y: &[f64]) -> Result<LinearModel, FitError> {
    let n = y.len();
    if x.len() != n { return Err(FitError::DimensionMismatch); }
    if n == 0 { return Err(FitError::EmptyData); }

    let n_features = x[0].len();
    let n_cols = n_features + 1; // +1 for intercept

    // Build design matrix X with prepended bias column
    let design_data: Vec<f64> = x.iter().flat_map(|row| {
        std::iter::once(1.0).chain(row.iter().copied())
    }).collect();
    let design = Matrix::from_vec(n, n_cols, design_data);

    let xt = design.transpose();
    let xtx = xt.matmul(&design).ok_or(FitError::SingularMatrix)?;
    let xty = xt.mul_vec(y).ok_or(FitError::DimensionMismatch)?;
    let (xtx_inv, _) = xtx.inverse().ok_or(FitError::SingularMatrix)?;
    let beta = xtx_inv.mul_vec(&xty).ok_or(FitError::SingularMatrix)?;

    // Predictions and R²
    let y_hat: Vec<f64> = x.iter().map(|row| {
        beta[0] + row.iter().zip(&beta[1..]).map(|(xi, bi)| xi * bi).sum::<f64>()
    }).collect();

    let residuals: Vec<f64> = y.iter().zip(&y_hat).map(|(yi, yhi)| yi - yhi).collect();
    let y_mean = mean(y).map_err(|_| FitError::EmptyData)?;
    let ss_res: f64 = residuals.iter().map(|r| r.powi(2)).sum();
    let ss_tot: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum();
    let r_squared = if ss_tot < 1e-12 { 1.0 } else { 1.0 - ss_res / ss_tot };

    Ok(LinearModel { coefficients: beta, r_squared, residuals })
}

/// Predict using a fitted model
pub fn predict(model: &LinearModel, x: &[f64]) -> f64 {
    model.coefficients[0] + x.iter().zip(&model.coefficients[1..]).map(|(xi, bi)| xi * bi).sum::<f64>()
}

#[derive(Debug, PartialEq)]
pub enum FitError {
    DimensionMismatch,
    SingularMatrix,
    EmptyData,
}

impl From<StatsError> for FitError {
    fn from(_: StatsError) -> Self { FitError::EmptyData }
}

impl std::fmt::Display for FitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FitError::DimensionMismatch => write!(f, "X rows must equal y length"),
            FitError::SingularMatrix    => write!(f, "design matrix is singular (linearly dependent features)"),
            FitError::EmptyData         => write!(f, "empty data"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(data: &[(f64, f64)]) -> Vec<Vec<f64>> {
        data.iter().map(|(x, _)| vec![*x]).collect()
    }
    fn ys(data: &[(f64, f64)]) -> Vec<f64> {
        data.iter().map(|(_, y)| *y).collect()
    }

    #[test]
    fn perfect_linear_fit_recovers_coefficients() {
        // y = 3 + 2x
        let data: Vec<(f64, f64)> = (0..10).map(|i| (i as f64, 3.0 + 2.0 * i as f64)).collect();
        let model = fit(&rows(&data), &ys(&data)).unwrap();
        assert!((model.coefficients[0] - 3.0).abs() < 1e-8, "intercept");
        assert!((model.coefficients[1] - 2.0).abs() < 1e-8, "slope");
        assert!((model.r_squared - 1.0).abs() < 1e-8, "R²");
    }

    #[test]
    fn r_squared_of_perfect_fit_is_one() {
        let data: Vec<(f64, f64)> = (1..=5).map(|i| (i as f64, (i * i) as f64)).collect();
        // Quadratic data won't be a perfect linear fit — R² should be < 1
        let model = fit(&rows(&data), &ys(&data)).unwrap();
        assert!(model.r_squared < 1.0);
    }

    #[test]
    fn predict_matches_fitted_values() {
        let data: Vec<(f64, f64)> = (0..5).map(|i| (i as f64, 3.0 + 2.0 * i as f64)).collect();
        let model = fit(&rows(&data), &ys(&data)).unwrap();
        for (x, y) in &data {
            let pred = predict(&model, &[*x]);
            assert!((pred - y).abs() < 1e-8, "pred {pred} != {y}");
        }
    }

    #[test]
    fn empty_data_returns_error() {
        assert_eq!(fit(&[], &[]).unwrap_err(), FitError::EmptyData);
    }

    #[test]
    fn mismatched_lengths_return_error() {
        let x = vec![vec![1.0], vec![2.0]];
        let y = vec![1.0];
        assert_eq!(fit(&x, &y).unwrap_err(), FitError::DimensionMismatch);
    }

    #[test]
    fn multi_feature_fit() {
        // y = 1 + 2x₁ + 3x₂  — x₁ and x₂ must be independent (not collinear)
        let x: Vec<Vec<f64>> = (0..20).map(|i| vec![i as f64, (i as f64).powi(2)]).collect();
        let y: Vec<f64> = x.iter().map(|r| 1.0 + 2.0 * r[0] + 3.0 * r[1]).collect();
        let model = fit(&x, &y).unwrap();
        assert!((model.coefficients[0] - 1.0).abs() < 1e-6, "β₀");
        assert!((model.coefficients[1] - 2.0).abs() < 1e-6, "β₁");
        assert!((model.coefficients[2] - 3.0).abs() < 1e-6, "β₂");
        assert!((model.r_squared - 1.0).abs() < 1e-8);
    }
}

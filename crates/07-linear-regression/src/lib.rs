// ============================================================
//  YOUR CHALLENGE - implement Ordinary Least Squares regression.
//
//  OLS via normal equations: beta = (X'X)^{-1} X'y
//  where X is the design matrix with a bias column of 1s prepended.
//
//  Steps:
//    1. Build design matrix: prepend a column of 1.0 to x
//    2. Compute X'X and X'y
//    3. Invert X'X (use matrix_math::Matrix)
//    4. Compute beta = (X'X)^{-1} * X'y
//    5. Compute residuals and R^2
//
//  Use matrix_math::Matrix for all matrix operations.
//  coefficients[0] is the intercept, coefficients[1..] are slopes.
// ============================================================

use matrix_math::Matrix;
use statistics_core::{mean, StatsError};

#[derive(Debug)]
pub struct LinearModel {
    /// Fitted coefficients: beta[0] = intercept, beta[1..] = feature weights
    pub coefficients: Vec<f64>,
    pub r_squared: f64,
    pub residuals: Vec<f64>,
}

/// Fit OLS via normal equations: beta = (X'X)^{-1}X'y
/// `x` is a 2D slice in row-major order (n_samples x n_features, no bias column).
/// A bias column of 1s is prepended automatically.
pub fn fit(x: &[Vec<f64>], y: &[f64]) -> Result<LinearModel, FitError> {
    todo!()
}

/// Predict using a fitted model
pub fn predict(model: &LinearModel, x: &[f64]) -> f64 {
    todo!()
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
        assert!((model.r_squared - 1.0).abs() < 1e-8, "R^2");
    }

    #[test]
    fn r_squared_of_perfect_fit_is_one() {
        let data: Vec<(f64, f64)> = (1..=5).map(|i| (i as f64, (i * i) as f64)).collect();
        // Quadratic data won't be a perfect linear fit - R^2 should be < 1
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
        // y = 1 + 2x1 + 3x2 - x1 and x2 must be independent (not collinear)
        let x: Vec<Vec<f64>> = (0..20).map(|i| vec![i as f64, (i as f64).powi(2)]).collect();
        let y: Vec<f64> = x.iter().map(|r| 1.0 + 2.0 * r[0] + 3.0 * r[1]).collect();
        let model = fit(&x, &y).unwrap();
        assert!((model.coefficients[0] - 1.0).abs() < 1e-6, "b0");
        assert!((model.coefficients[1] - 2.0).abs() < 1e-6, "b1");
        assert!((model.coefficients[2] - 3.0).abs() < 1e-6, "b2");
        assert!((model.r_squared - 1.0).abs() < 1e-8);
    }
}

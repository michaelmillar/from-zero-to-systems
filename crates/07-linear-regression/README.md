# linear-regression

> Ordinary least squares via normal equations - demand forecasting, A/B test analysis, and economic modelling.

## ELI5

Imagine you notice that ice cream sales go up when the temperature rises. Linear regression draws the best straight line through your data points - the one that minimises the total squared distance between the line and each point. Once you have that line, you can predict: "if tomorrow is 30°C, we'll probably sell about 400 ice creams." It works for one variable or many at once (temperature AND day-of-week AND price).

## For the Educated Generalist

**Ordinary Least Squares (OLS)** finds the coefficient vector β that minimises the sum of squared residuals: `‖y − Xβ‖²`. Setting the gradient to zero yields the **normal equations**: `XᵀXβ = Xᵀy`, solved as `β = (XᵀX)⁻¹Xᵀy`.

This is the closed-form solution - elegant but with two important caveats:

1. **Multicollinearity**: if any feature is a linear combination of others, `XᵀX` is singular and has no inverse. This is why the test suite verifies features are genuinely independent.
2. **Computational cost**: inverting an (n×n) matrix is O(n³). For large feature sets (thousands of features), iterative methods like gradient descent (crate 19) are preferred over the normal equations.

**R²** (coefficient of determination) measures the proportion of variance in y explained by the model. R² = 1 means a perfect fit; R² = 0 means the model does no better than predicting the mean. In practice, R² > 0.8 is considered a strong fit for noisy real-world data.

**Residual analysis** is just as important as R². Even a high-R² model can be wrong if residuals are non-random (indicating a non-linear relationship) or heteroscedastic (variance changes with x, violating OLS assumptions). This is why the `LinearModel` struct exposes residuals alongside coefficients.

## What it does

Fits a multivariate OLS model using the normal equations, computes R², and exposes residuals. The binary demonstrates demand forecasting: recovering temperature and price sensitivity coefficients from noisy simulated sales data.

## Used in the wild

- **Amazon demand forecasting** - linear models are the baseline against which every ML model is measured; often surprisingly competitive
- **academic econometrics** - OLS is the workhorse of empirical economics (wage equations, price elasticity, diff-in-diff A/B tests)
- **clinical research** - adjusted regression controls for confounders (age, sex, comorbidities) in observational studies
- **quantitative finance** - factor models (Fama-French 3-factor) are OLS regressions of returns on risk factors

## Run it

```bash
cargo run -p linear-regression
```

## Use it as a library

```rust
use linear_regression::{fit, predict};

let x = vec![vec![25.0, 0.0], vec![30.0, -3.0]]; // [temperature, price_delta]
let y = vec![900.0, 1050.0];
let model = fit(&x, &y).unwrap();
println!("R² = {:.4}", model.r_squared);
println!("Prediction: {:.0}", predict(&model, &[28.0, -1.0]));
```

## Rust concepts covered

- **Lifetimes in practice**: `fit` borrows `x` and `y` slices without copying - the borrow checker enforces that data outlives the computation
- **Custom error enum with `Display`**: `FitError` gives callers specific, actionable error variants rather than a generic string
- **`From` trait**: `impl From<StatsError> for FitError` enables `?` to convert between error types automatically
- **Builder-style API**: the `LinearModel` struct bundles coefficients, R², and residuals - avoiding multiple return values

## Builds on

- [`statistics-core`](../05-statistics-core/) - uses `mean()` for computing the y-bar needed for R² calculation
- [`matrix-math`](../06-matrix-math/) - the normal equations are solved using `Matrix::inverse()` and `Matrix::matmul()`

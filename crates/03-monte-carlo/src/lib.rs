// ============================================================
//  YOUR CHALLENGE - price options and calculate Value at Risk
//  using Monte Carlo simulation.
//
//  European call option price via geometric Brownian motion:
//    S_T = S_0 * exp((r - 0.5*sigma^2)*T + sigma*sqrt(T)*Z)
//    payoff = max(S_T - K, 0)
//    price  = e^(-rT) * mean(payoff)
//
//  Black-Scholes for validation:
//    d1 = (ln(S/K) + (r + 0.5*sigma^2)*T) / (sigma*sqrt(T))
//    d2 = d1 - sigma*sqrt(T)
//    price = S*N(d1) - K*e^(-rT)*N(d2)
//
//  Value at Risk (VaR): sort losses, return the loss at
//  the given confidence percentile.
//
//  Hint: use rayon's into_par_iter() for parallel chunks.
//        normal_cdf is already implemented below.
// ============================================================

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

/// A European call option: the right (not obligation) to buy an asset at `strike`
/// price at expiry. Priced via geometric Brownian motion.
pub struct EuropeanCall {
    pub spot: f64,       // current asset price (S0)
    pub strike: f64,     // exercise price (K)
    pub rate: f64,       // annualised risk-free rate (r)
    pub volatility: f64, // annualised volatility (sigma)
    pub expiry: f64,     // time to expiry in years (T)
}

/// Monte Carlo price for a European call option.
/// Simulates `trials` paths of geometric Brownian motion, computes payoffs,
/// and discounts back to present value.
pub fn price_european_call(option: &EuropeanCall, trials: u64, seed: u64) -> f64 {
    todo!()
}

/// Black-Scholes analytical price for a European call - used to validate MC results.
pub fn black_scholes_call(option: &EuropeanCall) -> f64 {
    todo!()
}

/// Value at Risk at a given confidence level (e.g. 0.95 for 95th percentile).
/// Input: a slice of per-period returns (negative = loss).
/// Returns: the loss not exceeded in `confidence` fraction of periods.
pub fn value_at_risk(returns: &[f64], confidence: f64) -> f64 {
    todo!()
}

/// Abramowitz & Stegun approximation of the standard normal CDF (error < 7.5e-8)
fn normal_cdf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let poly = t * (0.319381530
        + t * (-0.356563782
        + t * (1.781477937
        + t * (-1.821255978
        + t * 1.330274429))));
    let pdf = (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let cdf = 1.0 - pdf * poly;
    if x >= 0.0 { cdf } else { 1.0 - cdf }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atm_call() -> EuropeanCall {
        EuropeanCall { spot: 100.0, strike: 100.0, rate: 0.05, volatility: 0.20, expiry: 1.0 }
    }

    #[test]
    fn mc_option_price_within_1pct_of_black_scholes() {
        let opt = atm_call();
        let analytical = black_scholes_call(&opt);
        let mc = price_european_call(&opt, 500_000, 42);
        let error = ((mc - analytical) / analytical).abs();
        assert!(
            error < 0.01,
            "MC price {:.4} deviates {:.2}% from B-S {:.4}",
            mc, error * 100.0, analytical
        );
    }

    #[test]
    fn deep_out_of_money_option_is_near_zero() {
        let opt = EuropeanCall { spot: 100.0, strike: 200.0, rate: 0.05, volatility: 0.20, expiry: 1.0 };
        let price = price_european_call(&opt, 200_000, 7);
        assert!(price < 0.01, "deep OTM option priced at {:.6}", price);
    }

    #[test]
    fn deep_in_money_option_approaches_intrinsic_value() {
        let opt = EuropeanCall { spot: 200.0, strike: 100.0, rate: 0.05, volatility: 0.20, expiry: 1.0 };
        let price = price_european_call(&opt, 200_000, 7);
        // Intrinsic value lower bound: S - K*e^(-rT) approximately 100 + discount
        assert!(price > 95.0, "deep ITM option priced at {:.2}", price);
    }

    #[test]
    fn var_95_is_less_than_var_99() {
        let returns: Vec<f64> = (-100_i32..=100).map(|i| i as f64 * 0.01).collect();
        let var95 = value_at_risk(&returns, 0.95);
        let var99 = value_at_risk(&returns, 0.99);
        assert!(var95 < var99, "VaR95 {:.2} should be < VaR99 {:.2}", var95, var99);
    }

    #[test]
    fn var_of_all_gains_is_non_positive() {
        let returns: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let var = value_at_risk(&returns, 0.95);
        assert!(var <= 0.0, "all-gains portfolio has positive VaR: {:.2}", var);
    }
}

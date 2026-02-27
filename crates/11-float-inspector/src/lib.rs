// ============================================================
//  YOUR CHALLENGE - dissect IEEE 754 double-precision floats.
//
//  A 64-bit float is laid out in memory as:
//    [sign: 1 bit][exponent: 11 bits][mantissa: 52 bits]
//
//  The exponent is stored with a bias of 1023 (so stored 1023 = actual 0).
//  The mantissa has an implicit leading 1 bit (except for subnormals).
//
//  Hint: use `f64::to_bits()` / `f64::from_bits()` to get the raw u64.
//  Use `transmute` only if you need to inspect the byte pattern directly.
// ============================================================

/// Extract the sign bit: 0 = positive, 1 = negative.
pub fn sign_bit(x: f64) -> u64 {
    todo!()
}

/// Extract the raw 11-bit biased exponent (before subtracting bias of 1023).
pub fn raw_exponent(x: f64) -> u64 {
    todo!()
}

/// Extract the actual (unbiased) exponent: raw_exponent - 1023.
/// Returns None for special values (NaN, infinity, subnormals).
pub fn actual_exponent(x: f64) -> Option<i32> {
    todo!()
}

/// Extract the raw 52-bit mantissa (significand, without the implicit leading 1).
pub fn mantissa_bits(x: f64) -> u64 {
    todo!()
}

/// Distance in Units in the Last Place (ULPs) between two floats.
/// ULP distance is the number of representable floats between a and b.
/// Useful for comparing floats with meaningful tolerance.
pub fn ulp_distance(a: f64, b: f64) -> u64 {
    todo!()
}

/// Return true if |a - b| < epsilon OR ulp_distance(a, b) < max_ulps.
/// Combines absolute and relative comparison for robust float equality.
pub fn nearly_equal(a: f64, b: f64, epsilon: f64, max_ulps: u64) -> bool {
    todo!()
}

/// Demonstrate catastrophic cancellation: compute (x+1)^2 - x^2 - 2x - 1
/// which is mathematically 0 for all x, but numerically unstable for large x.
/// Returns the numerical error.
pub fn cancellation_error(x: f64) -> f64 {
    todo!()
}

// ============================================================
//  TESTS - they ARE the spec.
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod ieee754_structure {
        use super::*;

        #[test]
        fn sign_bit_of_positive_number_is_0() {
            assert_eq!(sign_bit(1.0), 0);
        }

        #[test]
        fn sign_bit_of_negative_number_is_1() {
            assert_eq!(sign_bit(-1.0), 1);
        }

        #[test]
        fn raw_exponent_of_1_is_bias_1023() {
            // 1.0 = 1 x 2^0, stored exponent = 0 + 1023 = 1023
            assert_eq!(raw_exponent(1.0), 1023);
        }

        #[test]
        fn actual_exponent_of_1_is_zero() {
            assert_eq!(actual_exponent(1.0), Some(0));
        }

        #[test]
        fn actual_exponent_of_8_is_3() {
            // 8.0 = 1 x 2^3
            assert_eq!(actual_exponent(8.0), Some(3));
        }

        #[test]
        fn actual_exponent_of_nan_is_none() {
            assert_eq!(actual_exponent(f64::NAN), None);
        }

        #[test]
        fn actual_exponent_of_infinity_is_none() {
            assert_eq!(actual_exponent(f64::INFINITY), None);
        }

        #[test]
        fn mantissa_of_1_has_no_fractional_bits() {
            // 1.0 = 1.0000...0 x 2^0, mantissa bits are all zero
            assert_eq!(mantissa_bits(1.0), 0);
        }
    }

    mod ulp_comparison {
        use super::*;

        #[test]
        fn ulp_distance_of_identical_values_is_zero() {
            assert_eq!(ulp_distance(1.0, 1.0), 0);
        }

        #[test]
        fn ulp_distance_of_adjacent_floats_is_one() {
            let a = 1.0_f64;
            let b = f64::from_bits(a.to_bits() + 1);
            assert_eq!(ulp_distance(a, b), 1);
        }

        #[test]
        fn nearly_equal_detects_close_values() {
            let a = 0.1 + 0.2;
            let b = 0.3;
            // Not bitwise equal but should be nearly equal
            assert!(nearly_equal(a, b, 1e-10, 4));
        }

        #[test]
        fn nearly_equal_rejects_distant_values() {
            assert!(!nearly_equal(1.0, 2.0, 1e-10, 4));
        }
    }

    mod numerical_stability {
        use super::*;

        #[test]
        fn cancellation_error_grows_with_magnitude() {
            let small = cancellation_error(1.0).abs();
            let large = cancellation_error(1e10).abs();
            assert!(large > small,
                "error at 1e10 ({large:e}) should exceed error at 1.0 ({small:e})");
        }

        #[test]
        fn cancellation_error_of_small_x_is_near_zero() {
            assert!(cancellation_error(1.0).abs() < 1e-10);
        }
    }
}

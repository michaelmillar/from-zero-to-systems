// ============================================================
//  YOUR CHALLENGE - implement the functions below.
//  Run `cargo test -p bit-manipulator` to see what's failing.
//  All functions have the correct signature; your job is to
//  replace every `todo!()` with a working implementation.
// ============================================================

/// Extract `len` bits from `value` starting at bit position `offset` (0 = LSB).
/// Example: extract_bits(0b10110110, offset=2, len=4) -> 0b1101 (bits 2..5)
pub fn extract_bits(value: u32, offset: u8, len: u8) -> u32 {
    todo!()
}

/// Set bit at position `bit` (0 = LSB) in `value`.
pub fn set_bit(value: u32, bit: u8) -> u32 {
    todo!()
}

/// Clear bit at position `bit` (0 = LSB) in `value`.
pub fn clear_bit(value: u32, bit: u8) -> u32 {
    todo!()
}

/// Toggle bit at position `bit` (0 = LSB) in `value`.
pub fn toggle_bit(value: u32, bit: u8) -> u32 {
    todo!()
}

/// Return true if bit at position `bit` (0 = LSB) is set.
pub fn is_bit_set(value: u32, bit: u8) -> bool {
    todo!()
}

/// Count the number of set bits (population count / Hamming weight).
/// Hint: Rust has a built-in method for this on integers.
pub fn count_ones(value: u32) -> u32 {
    todo!()
}

/// Compute parity: true if the number of set bits is odd.
pub fn parity(value: u32) -> bool {
    todo!()
}

/// Rotate bits left by `n` positions (wrapping, 32-bit).
pub fn rotate_left(value: u32, n: u8) -> u32 {
    todo!()
}

/// Rotate bits right by `n` positions (wrapping, 32-bit).
pub fn rotate_right(value: u32, n: u8) -> u32 {
    todo!()
}

/// Reverse the byte order of a u32 (big-endian <-> little-endian swap).
/// Hint: Rust has a built-in for this too.
pub fn swap_bytes(value: u32) -> u32 {
    todo!()
}

// ============================================================
//  IPv4 header field extraction - a real protocol use case.
//  An IPv4 header byte 0 is: [version: 4 bits][IHL: 4 bits]
//  Byte 8 is TTL; bytes 12-15 are the source IP address.
// ============================================================

/// Parse the IP version from byte 0 of an IPv4 header (upper 4 bits).
pub fn ipv4_version(header_byte0: u8) -> u8 {
    todo!()
}

/// Parse the Internet Header Length (IHL) from byte 0 (lower 4 bits).
/// IHL x 4 = header length in bytes.
pub fn ipv4_ihl(header_byte0: u8) -> u8 {
    todo!()
}

/// Parse a source IPv4 address from bytes 12-15 of a header slice.
/// Returns (octet0, octet1, octet2, octet3).
pub fn ipv4_src_addr(header: &[u8]) -> (u8, u8, u8, u8) {
    todo!()
}

// ============================================================
//  TESTS - written BDD-style.
//  Read these carefully: they ARE the specification.
//  You should also add your own tests for edge cases.
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- extract_bits ────────────────────────────────────────
    mod extracting_bits {
        use super::*;

        #[test]
        fn given_0b10110110_extract_bits_2_to_5_returns_0b1101() {
            assert_eq!(extract_bits(0b10110110, 2, 4), 0b1101);
        }

        #[test]
        fn given_any_value_extract_zero_bits_returns_zero() {
            assert_eq!(extract_bits(0xFFFFFFFF, 0, 0), 0);
        }

        #[test]
        fn given_0xdeadbeef_extract_all_32_bits_returns_value_unchanged() {
            assert_eq!(extract_bits(0xDEADBEEF, 0, 32), 0xDEADBEEF);
        }
    }

    // -- set / clear / toggle ────────────────────────────────
    mod setting_and_clearing_bits {
        use super::*;

        #[test]
        fn setting_bit_3_of_zero_returns_0b00001000() {
            assert_eq!(set_bit(0, 3), 0b00001000);
        }

        #[test]
        fn setting_a_bit_that_is_already_set_is_idempotent() {
            assert_eq!(set_bit(0xFF, 4), 0xFF);
        }

        #[test]
        fn clearing_bit_3_of_0b11111111_returns_0b11110111() {
            assert_eq!(clear_bit(0xFF, 3), 0b11110111);
        }

        #[test]
        fn clearing_a_bit_already_clear_is_idempotent() {
            assert_eq!(clear_bit(0, 5), 0);
        }

        #[test]
        fn toggling_bit_0_of_zero_sets_it() {
            assert_eq!(toggle_bit(0, 0), 1);
        }

        #[test]
        fn toggling_bit_0_twice_returns_original_value() {
            let original = 0b10101010_u32;
            assert_eq!(toggle_bit(toggle_bit(original, 0), 0), original);
        }
    }

    // -- is_bit_set ──────────────────────────────────────────
    mod checking_bits {
        use super::*;

        #[test]
        fn bit_is_set_returns_true_when_set() {
            assert!(is_bit_set(0b00010000, 4));
        }

        #[test]
        fn bit_is_set_returns_false_when_clear() {
            assert!(!is_bit_set(0b11101111, 4));
        }
    }

    // -- count_ones / parity ─────────────────────────────────
    mod population_count_and_parity {
        use super::*;

        #[test]
        fn count_ones_of_zero_is_zero() {
            assert_eq!(count_ones(0), 0);
        }

        #[test]
        fn count_ones_of_0xff_is_8() {
            assert_eq!(count_ones(0xFF), 8);
        }

        #[test]
        fn count_ones_of_0xdeadbeef_is_known_value() {
            assert_eq!(count_ones(0xDEADBEEF), 24);
        }

        #[test]
        fn parity_of_value_with_odd_bits_set_is_true() {
            assert!(parity(0b111)); // 3 bits set -> odd
        }

        #[test]
        fn parity_of_value_with_even_bits_set_is_false() {
            assert!(!parity(0b1111)); // 4 bits set -> even
        }
    }

    // -- rotate ──────────────────────────────────────────────
    mod rotation {
        use super::*;

        #[test]
        fn rotate_left_by_1_doubles_value_for_small_numbers() {
            assert_eq!(rotate_left(1, 1), 2);
        }

        #[test]
        fn rotate_left_by_32_returns_original() {
            assert_eq!(rotate_left(0xDEADBEEF, 32), 0xDEADBEEF);
        }

        #[test]
        fn rotate_right_is_inverse_of_rotate_left() {
            let val = 0xABCD1234_u32;
            for n in 0..=31 {
                assert_eq!(rotate_right(rotate_left(val, n), n), val);
            }
        }
    }

    // -- byte swap ───────────────────────────────────────────
    mod byte_order {
        use super::*;

        #[test]
        fn swap_bytes_of_0x01020304_returns_0x04030201() {
            assert_eq!(swap_bytes(0x01020304), 0x04030201);
        }

        #[test]
        fn swap_bytes_twice_returns_original() {
            assert_eq!(swap_bytes(swap_bytes(0xDEADBEEF)), 0xDEADBEEF);
        }
    }

    // -- IPv4 header parsing ─────────────────────────────────
    mod ipv4_parsing {
        use super::*;

        #[test]
        fn ipv4_version_from_standard_header_byte_is_4() {
            // 0x45 = 0b0100_0101: version=4, ihl=5
            assert_eq!(ipv4_version(0x45), 4);
        }

        #[test]
        fn ipv4_ihl_from_standard_header_byte_is_5() {
            assert_eq!(ipv4_ihl(0x45), 5);
        }

        #[test]
        fn ipv4_src_addr_parses_correctly() {
            let mut header = vec![0u8; 16];
            header[12] = 192;
            header[13] = 168;
            header[14] = 1;
            header[15] = 42;
            assert_eq!(ipv4_src_addr(&header), (192, 168, 1, 42));
        }
    }
}

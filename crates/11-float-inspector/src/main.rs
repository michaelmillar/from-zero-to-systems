use float_inspector::*;

fn print_float(label: &str, x: f64) {
    let bits = x.to_bits();
    let sign = sign_bit(x);
    let raw_exp = raw_exponent(x);
    let mant = mantissa_bits(x);
    let actual = actual_exponent(x).map(|e| format!("{e}")).unwrap_or("special".into());
    println!("  {label:<20} = {x:>14}");
    println!("    bits:     {bits:064b}");
    println!("    sign={sign}  raw_exp={raw_exp:<4} (actual={actual})  mantissa={mant:052b}");
    println!();
}

fn main() {
    println!("=== IEEE 754 Float Inspector ===\n");

    print_float("1.0",       1.0_f64);
    print_float("0.1 + 0.2", 0.1_f64 + 0.2);
    print_float("0.3",       0.3_f64);

    println!("  0.1 + 0.2 == 0.3?  {}", 0.1_f64 + 0.2 == 0.3);
    println!("  nearly_equal?      {}", nearly_equal(0.1 + 0.2, 0.3, 1e-10, 4));
    println!("  ULP distance:      {}", ulp_distance(0.1 + 0.2, 0.3));

    println!("\n=== Catastrophic Cancellation ===\n");
    println!("  (x+1)² - x² - 2x - 1  should always equal 0:\n");
    for x in [1.0, 1e6, 1e10, 1e14, 1e15] {
        let err = cancellation_error(x);
        println!("    x = {x:>10.0e}  →  error = {err:>12e}");
    }
    println!("\n  At x=1e15, the error is larger than 1 — the result is completely wrong.");
    println!("  This is how subtle floating-point bugs enter GPS, financial, and physics code.");
}

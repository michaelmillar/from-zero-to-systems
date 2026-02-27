use matrix_math::Matrix;

fn main() {
    // --- 2D rotation transform (robotics / graphics) ---
    println!("=== 2D Rotation Matrix (θ = 45°) ===\n");
    let theta = std::f64::consts::PI / 4.0;
    let rot = Matrix::from_vec(2, 2, vec![
        theta.cos(), -theta.sin(),
        theta.sin(),  theta.cos(),
    ]);
    println!("Rotation matrix:\n{}", rot);

    let point = vec![1.0, 0.0];
    let rotated = rot.mul_vec(&point).unwrap();
    println!("Rotating [1, 0] by 45° → [{:.4}, {:.4}]\n", rotated[0], rotated[1]);

    // --- Matrix inverse and determinant ---
    println!("=== Inverse & Determinant ===\n");
    let m = Matrix::from_vec(3, 3, vec![
        2.0, 1.0, 0.0,
        1.0, 3.0, 1.0,
        0.0, 1.0, 2.0,
    ]);
    println!("Matrix M:\n{}", m);
    let (inv, det) = m.inverse().unwrap();
    println!("det(M) = {:.4}", det);
    println!("M⁻¹:\n{}", inv);

    // Verify M × M⁻¹ ≈ I
    let should_be_identity = m.matmul(&inv).unwrap();
    println!("M × M⁻¹ (should be identity):\n{}", should_be_identity);

    // --- Normal equations preview (used by linear-regression crate) ---
    println!("=== Normal Equations: β = (XᵀX)⁻¹Xᵀy ===\n");
    // y = 2x + 3 data points: (1,5), (2,7), (3,9), (4,11)
    let x = Matrix::from_vec(4, 2, vec![
        1.0, 1.0,
        1.0, 2.0,
        1.0, 3.0,
        1.0, 4.0,
    ]);
    let y = vec![5.0, 7.0, 9.0, 11.0];
    let xt = x.transpose();
    let xtx = xt.matmul(&x).unwrap();
    let xty = xt.mul_vec(&y).unwrap();
    let (xtx_inv, _) = xtx.inverse().unwrap();
    let beta = xtx_inv.mul_vec(&xty).unwrap();
    println!("For y = 2x + 3: fitted β₀ = {:.4}, β₁ = {:.4}", beta[0], beta[1]);
}

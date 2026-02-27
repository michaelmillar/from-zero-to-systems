# matrix-math

> Matrix algebra from scratch — multiply, transpose, invert, determinant. The foundation for ML, robotics, computer graphics, and scientific computing.

## ELI5

A matrix is a grid of numbers. Multiplying two matrices together is a way of composing two transformations — like first rotating a shape, then stretching it. When you rotate a point on screen, your GPU is doing matrix multiplication millions of times per second. When a robot arm moves to a position, each joint's rotation is represented as a matrix, and they're all multiplied together to find where the end of the arm ends up. This crate builds all of that from scratch.

## For the Educated Generalist

**Matrix multiplication** is the composition of linear maps. If A represents "rotate by 30°" and B represents "scale by 2×", then A·B is "rotate then scale". This non-commutativity (A·B ≠ B·A in general) is what makes order matter in robotics and graphics pipelines.

**The inverse** of a matrix A is the matrix A⁻¹ such that A·A⁻¹ = I (identity). Not all matrices have inverses — a **singular** matrix (determinant = 0) represents a transformation that collapses space (e.g. projecting 3D onto a 2D plane), and is irreversible. Computing the inverse is central to solving linear systems Ax = b, which underlies ordinary least squares regression (crate 07).

**Gaussian elimination with partial pivoting** is the numerically stable algorithm used here. Partial pivoting (swapping rows to put the largest element in the pivot position) prevents catastrophic cancellation — a subtle source of floating-point error that naive elimination suffers from. This is what LAPACK (the library behind NumPy, MATLAB, and R's matrix operations) uses internally.

**Operator overloading** via Rust's `Mul` and `Add` traits lets us write `a * b` naturally, while the `Index` trait enables `m[(row, col)]` syntax. These are the same traits Rust's standard library uses for `Vec`, `String`, etc.

## What it does

Implements a row-major `Matrix` struct with multiply, add, transpose, inverse (Gaussian elimination with partial pivoting), and determinant. The binary demonstrates 2D rotation transforms and the normal equations preview for linear regression.

## Used in the wild

- **TensorFlow / PyTorch** — every neural network layer is a matrix multiplication; production systems use highly optimised BLAS/CUDA implementations of exactly this operation
- **OpenGL / WebGPU** — 4×4 transformation matrices (model, view, projection) are the foundation of all real-time 3D rendering
- **Boston Dynamics** — robot kinematics use homogeneous transformation matrices to compose joint rotations across a kinematic chain
- **NumPy** — `np.linalg.inv()` and `np.dot()` wrap LAPACK's `dgetrf`/`dgetri`, which implement the same Gaussian elimination

## Run it

```bash
cargo run -p matrix-math
```

## Use it as a library

```rust
use matrix_math::Matrix;

let a = Matrix::from_vec(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
let b = Matrix::from_vec(2, 2, vec![5.0, 6.0, 7.0, 8.0]);
let c = a.matmul(&b).unwrap();
let (inv, det) = c.inverse().unwrap();
println!("det = {:.2}", det);
```

## Rust concepts covered

- **Operator overloading**: implementing `Mul`, `Add`, `Index`, `IndexMut` — the same trait system that powers `+` on integers and `[]` on `Vec`
- **Row-major storage**: `data[r * cols + c]` — understanding memory layout is critical for cache performance and FFI with C libraries
- **`Option<T>` as error channel**: operations that can fail (incompatible dimensions, singular matrix) return `Option` rather than panicking
- **`fmt::Display`**: custom pretty-printing for the matrix — how Rust's formatting traits work
- **Partial pivoting**: a practical example of why naïve algorithms fail numerically, and how to fix them

## Builds on

Nothing — this is a standalone foundation used by crates 07, 11, 19, 20, 22, and 23.

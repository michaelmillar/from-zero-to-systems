// ============================================================
//  YOUR CHALLENGE - implement a Matrix type with linear algebra.
//
//  The Matrix struct stores data in row-major order (Vec<f64>).
//  Element (r, c) lives at index r * cols + c.
//
//  Implement:
//    - transpose: swap rows and columns
//    - matmul: matrix multiplication (return None if incompatible)
//    - mul_vec: multiply matrix by column vector
//    - inverse: Gaussian elimination with partial pivoting
//    - determinant: via inverse
//
//  The constructors (new, from_vec, identity) and index operators
//  are already provided - use self[(r, c)] in your implementations.
// ============================================================

use std::fmt;
use std::ops::{Add, Index, IndexMut, Mul};

#[derive(Clone, Debug, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols, data: vec![0.0; rows * cols] }
    }

    pub fn identity(n: usize) -> Self {
        let mut m = Self::new(n, n);
        for i in 0..n { m[(i, i)] = 1.0; }
        m
    }

    pub fn from_vec(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        assert_eq!(data.len(), rows * cols, "data length must equal rows x cols");
        Self { rows, cols, data }
    }

    pub fn transpose(&self) -> Self {
        todo!()
    }

    /// Matrix multiplication. Returns None if dimensions are incompatible.
    pub fn matmul(&self, rhs: &Matrix) -> Option<Matrix> {
        todo!()
    }

    /// Multiply matrix by column vector. Returns None if dimensions incompatible.
    pub fn mul_vec(&self, v: &[f64]) -> Option<Vec<f64>> {
        todo!()
    }

    /// Gaussian elimination with partial pivoting - returns (inverse, determinant).
    /// Returns None if matrix is singular or non-square.
    pub fn inverse(&self) -> Option<(Matrix, f64)> {
        todo!()
    }

    pub fn determinant(&self) -> Option<f64> {
        todo!()
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;
    fn index(&self, (r, c): (usize, usize)) -> &f64 {
        &self.data[r * self.cols + c]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut f64 {
        &mut self.data[r * self.cols + c]
    }
}

impl Add for Matrix {
    type Output = Option<Matrix>;
    fn add(self, rhs: Matrix) -> Option<Matrix> {
        if self.rows != rhs.rows || self.cols != rhs.cols { return None; }
        let data = self.data.iter().zip(&rhs.data).map(|(a, b)| a + b).collect();
        Some(Matrix { rows: self.rows, cols: self.cols, data })
    }
}

impl Mul for Matrix {
    type Output = Option<Matrix>;
    fn mul(self, rhs: Matrix) -> Option<Matrix> {
        self.matmul(&rhs)
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..self.rows {
            write!(f, "[")?;
            for c in 0..self.cols {
                if c > 0 { write!(f, ", ")?; }
                write!(f, "{:8.4}", self[(r, c)])?;
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_times_matrix_is_matrix() {
        let m = Matrix::from_vec(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        let i = Matrix::identity(2);
        assert_eq!(i.matmul(&m).unwrap(), m);
    }

    #[test]
    fn transpose_twice_is_identity() {
        let m = Matrix::from_vec(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(m.transpose().transpose(), m);
    }

    #[test]
    fn matrix_multiply_known_result() {
        let a = Matrix::from_vec(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        let b = Matrix::from_vec(2, 2, vec![5.0, 6.0, 7.0, 8.0]);
        let c = a.matmul(&b).unwrap();
        // [1*5+2*7, 1*6+2*8] = [19, 22]
        // [3*5+4*7, 3*6+4*8] = [43, 50]
        assert_eq!(c[(0, 0)], 19.0);
        assert_eq!(c[(0, 1)], 22.0);
        assert_eq!(c[(1, 0)], 43.0);
        assert_eq!(c[(1, 1)], 50.0);
    }

    #[test]
    fn inverse_of_identity_is_identity() {
        let i = Matrix::identity(3);
        let (inv, det) = i.inverse().unwrap();
        assert!((det - 1.0).abs() < 1e-10);
        for r in 0..3 {
            for c in 0..3 {
                let expected = if r == c { 1.0 } else { 0.0 };
                assert!((inv[(r, c)] - expected).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn matrix_times_its_inverse_is_identity() {
        let m = Matrix::from_vec(3, 3, vec![
            1.0, 2.0, 3.0,
            0.0, 1.0, 4.0,
            5.0, 6.0, 0.0,
        ]);
        let (inv, _) = m.inverse().unwrap();
        let prod = m.matmul(&inv).unwrap();
        for r in 0..3 {
            for c in 0..3 {
                let expected = if r == c { 1.0 } else { 0.0 };
                assert!((prod[(r, c)] - expected).abs() < 1e-9,
                    "prod[{r},{c}] = {} expected {}", prod[(r, c)], expected);
            }
        }
    }

    #[test]
    fn determinant_of_singular_is_none() {
        let m = Matrix::from_vec(2, 2, vec![1.0, 2.0, 2.0, 4.0]);
        assert!(m.determinant().is_none());
    }

    #[test]
    fn mul_vec_known_result() {
        let m = Matrix::from_vec(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        let v = vec![1.0, 1.0];
        assert_eq!(m.mul_vec(&v).unwrap(), vec![3.0, 7.0]);
    }

    #[test]
    fn incompatible_dimensions_return_none() {
        let a = Matrix::new(2, 3);
        let b = Matrix::new(2, 3);
        assert!(a.matmul(&b).is_none());
    }
}

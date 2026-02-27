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
        assert_eq!(data.len(), rows * cols, "data length must equal rows × cols");
        Self { rows, cols, data }
    }

    pub fn transpose(&self) -> Self {
        let mut out = Self::new(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                out[(c, r)] = self[(r, c)];
            }
        }
        out
    }

    /// Matrix multiplication. Returns None if dimensions are incompatible.
    pub fn matmul(&self, rhs: &Matrix) -> Option<Matrix> {
        if self.cols != rhs.rows { return None; }
        let mut out = Matrix::new(self.rows, rhs.cols);
        for i in 0..self.rows {
            for j in 0..rhs.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self[(i, k)] * rhs[(k, j)];
                }
                out[(i, j)] = sum;
            }
        }
        Some(out)
    }

    /// Multiply matrix by column vector. Returns None if dimensions incompatible.
    pub fn mul_vec(&self, v: &[f64]) -> Option<Vec<f64>> {
        if self.cols != v.len() { return None; }
        Some((0..self.rows).map(|r| {
            (0..self.cols).map(|c| self[(r, c)] * v[c]).sum()
        }).collect())
    }

    /// Gaussian elimination with partial pivoting — returns (inverse, determinant).
    /// Returns None if matrix is singular or non-square.
    pub fn inverse(&self) -> Option<(Matrix, f64)> {
        if self.rows != self.cols { return None; }
        let n = self.rows;
        let mut a = self.clone();
        let mut inv = Matrix::identity(n);
        let mut det = 1.0_f64;
        let mut sign = 1.0_f64;

        for col in 0..n {
            // Partial pivoting
            let pivot_row = (col..n)
                .max_by(|&r1, &r2| a[(r1, col)].abs().partial_cmp(&a[(r2, col)].abs()).unwrap())?;

            if a[(pivot_row, col)].abs() < 1e-12 { return None; } // singular

            if pivot_row != col {
                for j in 0..n {
                    let tmp = a[(col, j)]; a[(col, j)] = a[(pivot_row, j)]; a[(pivot_row, j)] = tmp;
                    let tmp = inv[(col, j)]; inv[(col, j)] = inv[(pivot_row, j)]; inv[(pivot_row, j)] = tmp;
                }
                sign *= -1.0;
            }

            det *= a[(col, col)];
            let pivot = a[(col, col)];
            for j in 0..n { a[(col, j)] /= pivot; inv[(col, j)] /= pivot; }

            for row in 0..n {
                if row == col { continue; }
                let factor = a[(row, col)];
                for j in 0..n {
                    let av = a[(col, j)]; a[(row, j)] -= factor * av;
                    let iv = inv[(col, j)]; inv[(row, j)] -= factor * iv;
                }
            }
        }
        Some((inv, sign * det))
    }

    pub fn determinant(&self) -> Option<f64> {
        self.inverse().map(|(_, d)| d)
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

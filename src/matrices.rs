#[cfg(feature = "no-std")]
use alloc::vec::Vec;

use std::ops::Mul;

use crate::{Element, Expression, Field};

/// A matrix of prime field elements.
pub struct ElementMatrix<F: Field> {
    rows: Vec<Vec<Element<F>>>,
}

impl<F: Field> ElementMatrix<F> {
    pub fn new(rows: Vec<Vec<Element<F>>>) -> Self {
        let num_cols = rows[0].len();
        for row in rows.iter() {
            assert_eq!(row.len(), num_cols, "Rows must have uniform length");
        }
        ElementMatrix { rows }
    }
}

impl<F: Field> Mul<&[Element<F>]> for &ElementMatrix<F> {
    type Output = Vec<Element<F>>;

    fn mul(self, rhs: &[Element<F>]) -> Self::Output {
        self.rows.iter().zip(rhs.iter())
            .map(|(row, val)| row.iter().fold(
                Element::zero(), |sum, row_i| sum + val * row_i))
            .collect()
    }
}

impl<F: Field> Mul<&[Expression<F>]> for &ElementMatrix<F> {
    type Output = Vec<Expression<F>>;

    fn mul(self, rhs: &[Expression<F>]) -> Self::Output {
        self.rows.iter().zip(rhs.iter())
            .map(|(row, val)| row.iter().fold(
                Expression::zero(), |sum, row_i| sum + val * row_i))
            .collect()
    }
}

impl<F: Field> Mul<&[Element<F>]> for ElementMatrix<F> {
    type Output = Vec<Element<F>>;

    fn mul(self, rhs: &[Element<F>]) -> Self::Output {
        &self * rhs
    }
}

impl<F: Field> Mul<&[Expression<F>]> for ElementMatrix<F> {
    type Output = Vec<Expression<F>>;

    fn mul(self, rhs: &[Expression<F>]) -> Self::Output {
        &self * rhs
    }
}

/// A Maximum Distance Separable matrix.
pub struct MdsMatrix<F: Field> {
    matrix: ElementMatrix<F>,
}

impl<F: Field> MdsMatrix<F> {
    pub fn new(rows: Vec<Vec<Element<F>>>) -> Self {
        // TODO: Verify the MDS diffusion property.
        MdsMatrix { matrix: ElementMatrix::new(rows) }
    }
}

impl<F: Field> Mul<&[Element<F>]> for &MdsMatrix<F> {
    type Output = Vec<Element<F>>;

    fn mul(self, rhs: &[Element<F>]) -> Self::Output {
        &self.matrix * rhs
    }
}

impl<F: Field> Mul<&[Expression<F>]> for &MdsMatrix<F> {
    type Output = Vec<Expression<F>>;

    fn mul(self, rhs: &[Expression<F>]) -> Self::Output {
        &self.matrix * rhs
    }
}

impl<F: Field> Mul<&[Element<F>]> for MdsMatrix<F> {
    type Output = Vec<Element<F>>;

    fn mul(self, rhs: &[Element<F>]) -> Self::Output {
        self.matrix * rhs
    }
}

impl<F: Field> Mul<&[Expression<F>]> for MdsMatrix<F> {
    type Output = Vec<Expression<F>>;

    fn mul(self, rhs: &[Expression<F>]) -> Self::Output {
        self.matrix * rhs
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn matrix_vector_multiplication() {
        // TODO
    }
}
//! This module extends GadgetBuilder with bitwise operations such as rotations, bitwise AND, and
//! so forth.

use core::borrow::Borrow;

use crate::gadget_builder::GadgetBuilder;
use crate::expression::Expression;
use crate::wire::Wire;
use crate::bits::BinaryExpression;

impl GadgetBuilder {
    /// ~x
    pub fn bitwise_not(&mut self, x: Vec<Wire>) -> Vec<Expression> {
        x.iter().map(|w| Expression::one() - Expression::from(w)).collect()
    }

    /// Rotate bits in the direction of greater significance.
    // TODO: Weird bit order issue...
    pub fn bitwise_rotate_left<BE: Borrow<BinaryExpression>>(&mut self, x: BE, n: usize)
                                                             -> BinaryExpression {
        let x = x.borrow();

        let l = x.len();
        let n_min = n % l;
        let bits = (0..l).map(|i| {
            if i >= n_min {
                x.bits[i - n_min].clone()
            } else {
                x.bits[i + l - n_min].clone()
            }
        }).collect();
        BinaryExpression { bits }
    }

    pub fn bitwise_and<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BinaryExpression
        where BE1: Borrow<BinaryExpression>, BE2: Borrow<BinaryExpression> {
        let x = x.borrow();
        let y = y.borrow();

        assert_eq!(x.len(), y.len());
        let l = x.len();
        let bits = (0..l).map(|i| {
            self.and(x.bits[i].clone(), y.bits[i].clone())
        }).collect();
        BinaryExpression { bits }
    }
}

#[cfg(test)]
mod tests {
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn bitwise_not() {
        let mut builder = GadgetBuilder::new();
        let x = builder.wire();
        builder.bitwise_not(vec![x]);
        let gadget = builder.build();

        let mut values = values!(x => 5.into());
        gadget.execute(&mut values);
    }
}
//! This module extends GadgetBuilder with bitwise operations such as rotations, bitwise AND, and
//! so forth.

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
    pub fn bitwise_rotate_left(&mut self, x: Vec<Wire>, n: usize) -> Vec<Wire> {
        let l = x.len();
        let n_min = n % l;
        (0..l).map(|i| {
            if i >= n_min {
                x[i - n_min]
            } else {
                x[i + l - n_min]
            }
        }).collect()
    }

    pub fn bitwise_and(&mut self, x: BinaryExpression, y: BinaryExpression) -> BinaryExpression {
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
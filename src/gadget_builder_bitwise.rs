//! This module extends GadgetBuilder with bitwise operations such as rotations, bitwise AND, and
//! so forth.

use core::borrow::Borrow;

use crate::gadget_builder::GadgetBuilder;
use crate::expression::BinaryExpression;

impl GadgetBuilder {
    /// ~x
    pub fn bitwise_not<BE: Borrow<BinaryExpression>>(&mut self, x: BE) -> BinaryExpression {
        let bits = x.borrow().bits.iter()
            .map(|w| self.not(w))
            .collect();
        BinaryExpression { bits }
    }

    /// Rotate bits in the direction of increasing significance. This is equivalent to "left rotate"
    /// in most programming languages.
    pub fn bitwise_rotate_dec_significance<BE: Borrow<BinaryExpression>>(&mut self, x: BE, n: usize)
                                                                         -> BinaryExpression {
        let x = x.borrow();
        let l = x.len();
        let bits = (0..l).map(|i| {
            let from_idx = (i + n) % l;
            x.bits[from_idx].clone()
        }).collect();
        BinaryExpression { bits }
    }

    /// Rotate bits in the direction of increasing significance. This is equivalent to "left rotate"
    /// in most programming languages.
    pub fn bitwise_rotate_inc_significance<BE: Borrow<BinaryExpression>>(&mut self, x: BE, n: usize)
                                                                         -> BinaryExpression {
        let x = x.borrow();
        let l = x.len();
        let bits = (0..l).map(|i| {
            // This is equivalent to (i - n) mod l.
            let from_idx = (l + i - n % l) % l;
            x.bits[from_idx].clone()
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
    use num::BigUint;

    use crate::expression::BinaryExpression;
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn bitwise_not() {
        let mut builder = GadgetBuilder::new();
        let x = builder.binary_wire(8);
        let not_x = builder.bitwise_not(BinaryExpression::from(&x));
        let gadget = builder.build();

        // ~00010011 = 11101100.
        let mut values = binary_unsigned_values!(x => BigUint::from(0b00010011u32));
        gadget.execute(&mut values);
        assert_eq!(BigUint::from(0b11101100u32), not_x.evaluate(&values));
    }

    #[test]
    fn bitwise_rotate_dec_significance() {
        let mut builder = GadgetBuilder::new();
        let x = builder.binary_wire(8);
        let x_rot = builder.bitwise_rotate_dec_significance(BinaryExpression::from(&x), 3);
        let gadget = builder.build();

        // 00000000 >> 3 = 00000000.
        let mut values_zero = binary_unsigned_values!(&x => BigUint::from(0u32));
        gadget.execute(&mut values_zero);
        assert_eq!(BigUint::from(0u32), x_rot.evaluate(&values_zero));

        // 00010011 >> 3 = 01100010.
        let mut values_nonzero = binary_unsigned_values!(x => BigUint::from(0b00010011u32));
        gadget.execute(&mut values_nonzero);
        assert_eq!(BigUint::from(0b01100010u32), x_rot.evaluate(&values_nonzero));
    }

    #[test]
    fn bitwise_rotate_dec_significance_multiple_wraps() {
        let mut builder = GadgetBuilder::new();
        let x = builder.binary_wire(8);
        let x_rot = builder.bitwise_rotate_dec_significance(BinaryExpression::from(&x), 19);
        let gadget = builder.build();

        // 00010011 >> 19 = 00010011 >> 3 = 01100010.
        let mut values = binary_unsigned_values!(x => BigUint::from(0b00010011u32));
        gadget.execute(&mut values);
        assert_eq!(BigUint::from(0b01100010u32), x_rot.evaluate(&values));
    }
}
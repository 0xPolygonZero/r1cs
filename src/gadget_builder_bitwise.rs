//! This module extends GadgetBuilder with bitwise operations such as rotations, bitwise AND, and
//! so forth.

use core::borrow::Borrow;

use crate::expression::BinaryExpression;
use crate::gadget_builder::GadgetBuilder;
use crate::field::Field;

impl<F: Field> GadgetBuilder<F> {
    /// ~x
    pub fn bitwise_not<BE>(&mut self, x: BE) -> BinaryExpression<F>
        where BE: Borrow<BinaryExpression<F>> {
        let bits = x.borrow().bits.iter()
            .map(|w| self.not(w))
            .collect();
        BinaryExpression { bits }
    }

    /// x & y
    pub fn bitwise_and<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BinaryExpression<F>
        where BE1: Borrow<BinaryExpression<F>>, BE2: Borrow<BinaryExpression<F>> {
        let x = x.borrow();
        let y = y.borrow();

        assert_eq!(x.len(), y.len());
        let l = x.len();
        let bits = (0..l).map(|i|
            self.and(&x.bits[i], &y.bits[i])
        ).collect();
        BinaryExpression { bits }
    }

    /// x | y
    pub fn bitwise_or<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BinaryExpression<F>
        where BE1: Borrow<BinaryExpression<F>>, BE2: Borrow<BinaryExpression<F>> {
        let x = x.borrow();
        let y = y.borrow();

        assert_eq!(x.len(), y.len());
        let l = x.len();
        let bits = (0..l).map(|i|
            self.or(&x.bits[i], &y.bits[i])
        ).collect();
        BinaryExpression { bits }
    }

    /// x | y
    pub fn bitwise_xor<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BinaryExpression<F>
        where BE1: Borrow<BinaryExpression<F>>, BE2: Borrow<BinaryExpression<F>> {
        let x = x.borrow();
        let y = y.borrow();

        assert_eq!(x.len(), y.len());
        let l = x.len();
        let bits = (0..l).map(|i|
            self.xor(&x.bits[i], &y.bits[i])
        ).collect();
        BinaryExpression { bits }
    }

    /// Rotate bits in the direction of increasing significance. This is equivalent to "left rotate"
    /// in most programming languages.
    pub fn bitwise_rotate_dec_significance<BE>(&mut self, x: BE, n: usize) -> BinaryExpression<F>
        where BE: Borrow<BinaryExpression<F>> {
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
    pub fn bitwise_rotate_inc_significance<BE>(&mut self, x: BE, n: usize) -> BinaryExpression<F>
        where BE: Borrow<BinaryExpression<F>> {
        let x = x.borrow();
        let l = x.len();
        let bits = (0..l).map(|i| {
            // This is equivalent to (i - n) mod l.
            let from_idx = (l + i - n % l) % l;
            x.bits[from_idx].clone()
        }).collect();
        BinaryExpression { bits }
    }
}

#[cfg(test)]
mod tests {
    use num::BigUint;

    use crate::expression::BinaryExpression;
    use crate::gadget_builder::GadgetBuilder;
    use crate::field::Bn128;

    #[test]
    fn bitwise_not() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.binary_wire(8);
        let not_x = builder.bitwise_not(BinaryExpression::from(&x));
        let gadget = builder.build();

        // ~00010011 = 11101100.
        let mut values = binary_unsigned_values!(x => BigUint::from(0b00010011u32));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(0b11101100u32), not_x.evaluate(&values));
    }

    #[test]
    fn bitwise_and() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.binary_wire(8);
        let y = builder.binary_wire(8);
        let x_and_y = builder.bitwise_and(BinaryExpression::from(&x), BinaryExpression::from(&y));
        let gadget = builder.build();

        // 0 & 0 = 0.
        let mut values_0_0 = binary_unsigned_values!(
            &x => BigUint::from(0u32),
            &y => BigUint::from(0u32));
        assert!(gadget.execute(&mut values_0_0));
        assert_eq!(BigUint::from(0u32), x_and_y.evaluate(&values_0_0));

        // 255 & 0 = 0.
        let mut values_255_0 = binary_unsigned_values!(
            &x => BigUint::from(0b11111111u32),
            &y => BigUint::from(0u32));
        assert!(gadget.execute(&mut values_255_0));
        assert_eq!(BigUint::from(0u32), x_and_y.evaluate(&values_255_0));

        // 255 & 255 = 255.
        let mut values_255_255 = binary_unsigned_values!(
            &x => BigUint::from(0b11111111u32),
            &y => BigUint::from(0b11111111u32));
        assert!(gadget.execute(&mut values_255_255));
        assert_eq!(BigUint::from(0b11111111u32), x_and_y.evaluate(&values_255_255));

        // 11111100 & 00111111 = 00111100.
        let mut values_11111100_00111111 = binary_unsigned_values!(
            &x => BigUint::from(0b11111100u32),
            &y => BigUint::from(0b00111111u32));
        assert!(gadget.execute(&mut values_11111100_00111111));
        assert_eq!(BigUint::from(0b00111100u32), x_and_y.evaluate(&values_11111100_00111111));
    }

    #[test]
    fn bitwise_rotate_dec_significance() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.binary_wire(8);
        let x_rot = builder.bitwise_rotate_dec_significance(BinaryExpression::from(&x), 3);
        let gadget = builder.build();

        // 00000000 >> 3 = 00000000.
        let mut values_zero = binary_unsigned_values!(&x => BigUint::from(0u32));
        assert!(gadget.execute(&mut values_zero));
        assert_eq!(BigUint::from(0u32), x_rot.evaluate(&values_zero));

        // 00010011 >> 3 = 01100010.
        let mut values_nonzero = binary_unsigned_values!(x => BigUint::from(0b00010011u32));
        assert!(gadget.execute(&mut values_nonzero));
        assert_eq!(BigUint::from(0b01100010u32), x_rot.evaluate(&values_nonzero));
    }

    #[test]
    fn bitwise_rotate_dec_significance_multiple_wraps() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.binary_wire(8);
        let x_rot = builder.bitwise_rotate_dec_significance(BinaryExpression::from(&x), 19);
        let gadget = builder.build();

        // 00010011 >> 19 = 00010011 >> 3 = 01100010.
        let mut values = binary_unsigned_values!(x => BigUint::from(0b00010011u32));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(0b01100010u32), x_rot.evaluate(&values));
    }
}
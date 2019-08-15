use core::borrow::Borrow;

use crate::expression::BinaryExpression;
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;
use crate::wire_values::WireValues;

impl<F: Field> GadgetBuilder<F> {
    /// Add two binary values in a widening manner. The result will be one bit longer than the
    /// longer of the two inputs.
    pub fn binary_sum<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BinaryExpression<F>
        where BE1: Borrow<BinaryExpression<F>>, BE2: Borrow<BinaryExpression<F>> {
        // We will non-deterministically generate the sum bits, join the three binary expressions,
        // and verify the summation on those field elements.

        let x = x.borrow();
        let y = y.borrow();

        let in_bits = x.len().max(y.len());
        let sum_bits = in_bits + 1;

        // TODO: Generalize this addition function to support larger operands.
        // We can split the bits into chunks and perform grade school addition on joined chunks.
        assert!(sum_bits < Element::<F>::max_bits(),
                "Binary operands are too large to fit an a field element.");

        let sum_wire = self.binary_wire(sum_bits);
        let sum = BinaryExpression::from(&sum_wire);

        let x_joined = x.join();
        let y_joined = y.join();
        let sum_joined = sum.join();

        self.assert_equal(&x_joined + &y_joined, sum_joined);

        self.generator(
            [x.dependencies(), y.dependencies()].concat(),
            move |values: &mut WireValues<F>| {
                let sum_element = (&x_joined + &y_joined).evaluate(values);
                let sum_biguint = sum_element.to_biguint();
                values.set_binary_unsigned(sum_wire.clone(), sum_biguint);
            },
        );

        sum
    }

    /// Add two binary values, ignoring any overflow.
    pub fn binary_sum_ignoring_overflow<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BinaryExpression<F>
        where BE1: Borrow<BinaryExpression<F>>, BE2: Borrow<BinaryExpression<F>> {
        let mut sum = self.binary_sum(x, y);
        sum.truncate(sum.len() - 1);
        sum
    }

    /// Add two binary values while asserting that overflow does not occur.
    pub fn binary_sum_asserting_no_overflow<BE1, BE2>(&mut self, x: BE1, y: BE2)
                                                      -> BinaryExpression<F>
        where BE1: Borrow<BinaryExpression<F>>, BE2: Borrow<BinaryExpression<F>> {
        let mut sum = self.binary_sum(x, y);
        let overflow_bit = &sum.bits[sum.len() - 1];
        self.assert_false(overflow_bit);
        sum.truncate(sum.len() - 1);
        sum
    }
}

#[cfg(test)]
mod tests {
    use num::BigUint;

    use crate::expression::BinaryExpression;
    use crate::field::Bn128;
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn binary_sum() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.binary_wire(4);
        let y = builder.binary_wire(4);
        let sum = builder.binary_sum(BinaryExpression::from(&x), BinaryExpression::from(&y));
        let gadget = builder.build();

        // 10 + 3 = 13.
        let mut values = binary_unsigned_values!(
            &x => BigUint::from(10u8), &y => BigUint::from(3u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(13u8), sum.evaluate(&values));

        // 10 + 11 = 21.
        let mut values = binary_unsigned_values!(
            &x => BigUint::from(10u8), &y => BigUint::from(11u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(21u8), sum.evaluate(&values));
    }

    #[test]
    fn binary_sum_ignoring_overflow() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.binary_wire(4);
        let y = builder.binary_wire(4);
        let sum = builder.binary_sum_ignoring_overflow(
            BinaryExpression::from(&x), BinaryExpression::from(&y));
        let gadget = builder.build();

        // 10 + 3 = 13.
        let mut values = binary_unsigned_values!(
            &x => BigUint::from(10u8), &y => BigUint::from(3u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(13u8), sum.evaluate(&values));

        // 10 + 11 = 21 % 16 = 5.
        let mut values = binary_unsigned_values!(
            &x => BigUint::from(10u8), &y => BigUint::from(11u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(5u8), sum.evaluate(&values));
    }

    #[test]
    fn binary_sum_asserting_no_overflow() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.binary_wire(4);
        let y = builder.binary_wire(4);
        let sum = builder.binary_sum_asserting_no_overflow(
            BinaryExpression::from(&x), BinaryExpression::from(&y));
        let gadget = builder.build();

        // 10 + 3 = 13.
        let mut values = binary_unsigned_values!(
            &x => BigUint::from(10u8), &y => BigUint::from(3u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(13u8), sum.evaluate(&values));

        // 10 + 11 = [error].
        let mut values = binary_unsigned_values!(
            &x => BigUint::from(10u8), &y => BigUint::from(11u8));
        assert!(!gadget.execute(&mut values));
    }
}
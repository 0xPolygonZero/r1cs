use itertools::Itertools;
use num::BigUint;
use num_traits::{One, Zero};

use crate::expression::{BinaryExpression, Expression};
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;
use crate::wire_values::WireValues;

impl<F: Field> GadgetBuilder<F> {
    /// Add two binary expressions in a widening manner. The result will be one bit longer than the
    /// longer of the two inputs.
    pub fn binary_sum(
        &mut self, x: &BinaryExpression<F>, y: &BinaryExpression<F>,
    ) -> BinaryExpression<F> {
        self.binary_summation(&[x.clone(), y.clone()])
    }

    /// Add two binary expressions, ignoring any overflow.
    pub fn binary_sum_ignoring_overflow(
        &mut self, x: &BinaryExpression<F>, y: &BinaryExpression<F>,
    ) -> BinaryExpression<F> {
        self.binary_summation_ignoring_overflow(&[x.clone(), y.clone()])
    }

    /// Add two binary expressions while asserting that overflow does not occur.
    pub fn binary_sum_asserting_no_overflow(
        &mut self, x: &BinaryExpression<F>, y: &BinaryExpression<F>,
    ) -> BinaryExpression<F> {
        self.binary_summation_asserting_no_overflow(&[x.clone(), y.clone()])
    }

    /// Add an arbitrary number of binary expressions. The result will be one bit longer than the
    /// longest input.
    pub fn binary_summation(&mut self, terms: &[BinaryExpression<F>]) -> BinaryExpression<F> {
        // We will non-deterministically generate the sum bits, join the binary expressions, and
        // verify the summation on those field elements.

        let mut max_sum = BigUint::zero();
        for term in terms {
            let max_term = (BigUint::one() << term.len()) - BigUint::one();
            max_sum += max_term;
        }
        let sum_bits = max_sum.bits();

        // TODO: Generalize this addition function to support larger operands.
        // We can split the bits into chunks and perform grade school addition on joined chunks.
        assert!(sum_bits < Element::<F>::max_bits(),
                "Binary operands are too large to fit an a field element.");

        let sum_wire = self.binary_wire(sum_bits);
        let sum = BinaryExpression::from(&sum_wire);

        let sum_of_terms = Expression::sum_of_expressions(
            &terms.iter().map(BinaryExpression::join).collect_vec());
        self.assert_equal(&sum_of_terms, &sum.join());

        self.generator(
            sum_of_terms.dependencies(),
            move |values: &mut WireValues<F>| {
                let sum_element = sum_of_terms.evaluate(values);
                let sum_biguint = sum_element.to_biguint();
                values.set_binary_unsigned(&sum_wire, sum_biguint);
            },
        );

        sum
    }

    /// Add an arbitrary number of binary expressions, ignoring any overflow.
    pub fn binary_summation_ignoring_overflow(&mut self, terms: &[BinaryExpression<F>])
                                              -> BinaryExpression<F> {
        let input_bits = terms.iter().fold(0, |x, y| x.max(y.len()));
        let mut sum = self.binary_summation(terms);
        sum.truncate(input_bits);
        sum
    }

    /// Add an arbitrary number of binary expressions, asserting that overflow does not occur.
    pub fn binary_summation_asserting_no_overflow(&mut self, terms: &[BinaryExpression<F>])
                                                  -> BinaryExpression<F> {
        let input_bits = terms.iter().fold(0, |x, y| x.max(y.len()));
        let mut sum = self.binary_summation(terms);
        let carry = BinaryExpression { bits: sum.bits[input_bits..].to_vec() };
        self.binary_assert_zero(&carry);
        sum.truncate(input_bits);
        sum
    }

    /// Assert that a binary expression is zero.
    pub fn binary_assert_zero(&mut self, x: &BinaryExpression<F>) {
        // TODO: Generalize to work with binary expressions larger than |F|.
        self.assert_zero(&x.join())
    }
}

#[cfg(test)]
mod tests {
    use num::BigUint;

    use crate::expression::BinaryExpression;
    use crate::gadget_builder::GadgetBuilder;
    use crate::test_util::F257;

    #[test]
    fn binary_sum() {
        let mut builder = GadgetBuilder::<F257>::new();
        let x = builder.binary_wire(4);
        let y = builder.binary_wire(4);
        let sum = builder.binary_sum(&BinaryExpression::from(&x), &BinaryExpression::from(&y));
        let gadget = builder.build();

        // 10 + 3 = 13.
        let mut values = binary_unsigned_values!(
            &x => &BigUint::from(10u8), &y => &BigUint::from(3u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(13u8), sum.evaluate(&values));

        // 10 + 11 = 21.
        let mut values = binary_unsigned_values!(
            &x => &BigUint::from(10u8), &y => &BigUint::from(11u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(21u8), sum.evaluate(&values));
    }

    #[test]
    fn binary_sum_ignoring_overflow() {
        let mut builder = GadgetBuilder::<F257>::new();
        let x = builder.binary_wire(4);
        let y = builder.binary_wire(4);
        let sum = builder.binary_sum_ignoring_overflow(
            &BinaryExpression::from(&x), &BinaryExpression::from(&y));
        let gadget = builder.build();

        // 10 + 3 = 13.
        let mut values = binary_unsigned_values!(
            &x => &BigUint::from(10u8), &y => &BigUint::from(3u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(13u8), sum.evaluate(&values));

        // 10 + 11 = 21 % 16 = 5.
        let mut values = binary_unsigned_values!(
            &x => &BigUint::from(10u8), &y => &BigUint::from(11u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(5u8), sum.evaluate(&values));
    }

    #[test]
    fn binary_sum_asserting_no_overflow() {
        let mut builder = GadgetBuilder::<F257>::new();
        let x = builder.binary_wire(4);
        let y = builder.binary_wire(4);
        let sum = builder.binary_sum_asserting_no_overflow(
            &BinaryExpression::from(&x), &BinaryExpression::from(&y));
        let gadget = builder.build();

        // 10 + 3 = 13.
        let mut values = binary_unsigned_values!(
            &x => &BigUint::from(10u8), &y => &BigUint::from(3u8));
        assert!(gadget.execute(&mut values));
        assert_eq!(BigUint::from(13u8), sum.evaluate(&values));

        // 10 + 11 = [error].
        let mut values = binary_unsigned_values!(
            &x => &BigUint::from(10u8), &y => &BigUint::from(11u8));
        assert!(!gadget.execute(&mut values));
    }

    // TODO: Test inputs with differing lengths.
    // TODO: Test summations with more than two terms.
}
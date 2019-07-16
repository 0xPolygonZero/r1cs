use core::borrow::Borrow;

use crate::expression::BinaryExpression;
use crate::field_element::FieldElement;
use crate::gadget_builder::GadgetBuilder;
use crate::wire_values::WireValues;

impl GadgetBuilder {
    /// Add two binary values in a widening manner. The result will be one bit longer than the
    /// longer of the two inputs.
    pub fn binary_sum<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BinaryExpression
        where BE1: Borrow<BinaryExpression>, BE2: Borrow<BinaryExpression> {
        // We will non-deterministically generate the sum bits, join the three binary expressions,
        // and verify the summation on those field elements.

        let x = x.borrow();
        let y = y.borrow();

        let in_bits = x.len().max(y.len());
        let sum_bits = in_bits + 1;

        // TODO: Generalize this addition function to support larger operands.
        // We can split the bits into chunks and perform grade school addition on joined chunks.
        assert!(sum_bits < FieldElement::max_bits(),
                "Binary operands are too large to fit an a field element.");

        let sum_wire = self.binary_wire(sum_bits);
        let sum = BinaryExpression::from(&sum_wire);

        let x_joined = x.join();
        let y_joined = y.join();
        let sum_joined = sum.join();

        self.assert_equal(&x_joined + &y_joined, sum_joined);

        self.generator(
            [x.dependencies(), y.dependencies()].concat(),
            move |values: &mut WireValues| {
                let sum_value = (&x_joined + &y_joined).evaluate(values).value().clone();
                values.set_binary_unsigned(sum_wire.clone(), sum_value);
            },
        );

        sum
    }

    /// Add two binary values, ignoring any overflow.
    pub fn binary_sum_ignoring_overflow<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BinaryExpression
        where BE1: Borrow<BinaryExpression>, BE2: Borrow<BinaryExpression> {
        let sum = self.binary_sum(x, y);
        sum.truncated(sum.len() - 1)
    }

    /// Add two binary values while asserting that overflow does not occur.
    pub fn binary_sum_asserting_no_overflow<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BinaryExpression
        where BE1: Borrow<BinaryExpression>, BE2: Borrow<BinaryExpression> {
        let sum = self.binary_sum(x, y);
        let overflow_bit = &sum.bits[sum.len() - 1];
        self.assert_false(overflow_bit);
        sum.truncated(sum.len() - 1)
    }
}

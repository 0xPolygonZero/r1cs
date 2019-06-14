use crate::bits::BinaryExpression;
use crate::gadget_builder::GadgetBuilder;

impl GadgetBuilder {
    /// Add two binary values in a widening manner. The result will be one bit longer than the
    /// longer of the two inputs.
    pub fn binary_sum(&mut self, x: BinaryExpression, y: BinaryExpression) -> BinaryExpression {
        let l = x.len().max(y.len());
        unimplemented!("TODO")
    }

    /// Add two binary values, ignoring any overflow.
    pub fn binary_sum_ignoring_overflow(&mut self, x: BinaryExpression, y: BinaryExpression) -> BinaryExpression {
        // TODO: Not optimal; bit of unused computation
        let sum = self.binary_sum(x, y);
        sum.truncated(sum.len() - 1)
    }

    /// Add two binary values while asserting that overflow does not occur.
    pub fn binary_sum_asserting_no_overflow(&mut self, x: BinaryExpression, y: BinaryExpression) -> BinaryExpression {
        let sum = self.binary_sum(x, y);
        let overflow_bit = sum.bits[sum.len() - 1].clone();
        self.assert_false(overflow_bit);
        sum.truncated(sum.len() - 1)
    }
}

//! This module extends GadgetBuilder with methods for splitting field elements into bits.

use core::borrow::Borrow;

use crate::expression::{BinaryExpression, Expression};
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;
use crate::wire_values::WireValues;

impl<F: Field> GadgetBuilder<F> {
    /// Split an arbitrary field element `x` into its canonical binary representation.
    pub fn split<E: Borrow<Expression<F>>>(&mut self, x: E) -> BinaryExpression<F> {
        let result = self.split_without_range_check(x, Element::<F>::max_bits());
        self.assert_lt_binary(&result, BinaryExpression::from(F::order()));
        result
    }

    /// Split an arbitrary field element `x` into a binary representation. Unlike `split`, this
    /// method permits two distinct binary decompositions: the canonical one, and another
    /// representation where the weighted sum of bits overflows the field size. This minimizes
    /// constraints, but the ambiguity can be a security problem depending on the context. If in
    /// doubt, use `split` instead.
    pub fn split_allowing_ambiguity<E>(&mut self, x: E) -> BinaryExpression<F>
        where E: Borrow<Expression<F>> {
        self.split_without_range_check(x, Element::<F>::max_bits())
    }

    /// Split `x` into `bits` bit wires. This method assumes `x < 2^bits < |F|`. Note that only one
    /// binary representation is possible here, since `bits` bits is not enough to overflow the
    /// field size.
    pub fn split_bounded<E>(&mut self, x: E, bits: usize) -> BinaryExpression<F>
        where E: Borrow<Expression<F>> {
        assert!(bits < Element::<F>::max_bits());
        self.split_without_range_check(x, bits)
    }

    fn split_without_range_check<E: Borrow<Expression<F>>>(&mut self, x: E, bits: usize)
                                                           -> BinaryExpression<F> {
        let x = x.borrow();
        let binary_wire = self.binary_wire(bits);
        let binary_exp = BinaryExpression::from(&binary_wire);
        let weighted_sum = binary_exp.join_allowing_overflow();
        self.assert_equal(x, weighted_sum);

        let x = x.clone();
        self.generator(
            x.dependencies(),
            move |values: &mut WireValues<F>| {
                let value = x.evaluate(values);
                assert!(value.bits() <= bits);
                for i in 0..bits {
                    values.set_boolean(binary_wire.bits[i], value.bit(i));
                }
            },
        );

        binary_exp
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::Bn128;
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn split_19_32() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let wire = builder.wire();
        let bit_wires = builder.split_bounded(Expression::from(wire), 32);
        let gadget = builder.build();

        let mut wire_values = values!(wire => 19u8.into());
        assert!(gadget.execute(&mut wire_values));

        assert_eq!(true, bit_wires.bits[0].evaluate(&wire_values));
        assert_eq!(true, bit_wires.bits[1].evaluate(&wire_values));
        assert_eq!(false, bit_wires.bits[2].evaluate(&wire_values));
        assert_eq!(false, bit_wires.bits[3].evaluate(&wire_values));
        assert_eq!(true, bit_wires.bits[4].evaluate(&wire_values));
        assert_eq!(false, bit_wires.bits[5].evaluate(&wire_values));
        assert_eq!(false, bit_wires.bits[6].evaluate(&wire_values));
    }
}
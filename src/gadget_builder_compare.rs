//! This module extends GadgetBuilder with methods for comparing native field elements.

use itertools::enumerate;

use crate::field_element::FieldElement;
use crate::gadget_builder::GadgetBuilder;
use crate::expression::Expression;
use crate::wire_values::WireValues;
use crate::bits::{BinaryExpression, BooleanExpression};

impl GadgetBuilder {
    /// Assert that x < y.
    pub fn assert_lt(&mut self, x: Expression, y: Expression) {
        let lt = self.lt(x, y);
        self.assert_true(lt);
    }

    /// Assert that x <= y.
    pub fn assert_le(&mut self, x: Expression, y: Expression) {
        let le = self.le(x, y);
        self.assert_true(le);
    }

    /// Assert that x > y.
    pub fn assert_gt(&mut self, x: Expression, y: Expression) {
        let gt = self.gt(x, y);
        self.assert_true(gt);
    }

    /// Assert that x >= y.
    pub fn assert_ge(&mut self, x: Expression, y: Expression) {
        let ge = self.ge(x, y);
        self.assert_true(ge);
    }

    /// x < y
    pub fn lt(&mut self, x: Expression, y: Expression) -> BooleanExpression {
        self.cmp(x, y, true, true)
    }

    /// x <= y
    pub fn le(&mut self, x: Expression, y: Expression) -> BooleanExpression {
        self.cmp(x, y, true, false)
    }

    /// x > y
    pub fn gt(&mut self, x: Expression, y: Expression) -> BooleanExpression {
        self.cmp(x, y, false, true)
    }

    /// x >= y
    pub fn ge(&mut self, x: Expression, y: Expression) -> BooleanExpression {
        self.cmp(x, y, false, false)
    }

    fn cmp(&mut self, x: Expression, y: Expression,
           less: bool, strict: bool) -> BooleanExpression {
        let bits = FieldElement::max_bits();
        let x_bits = self.split(x.into(), bits);
        let y_bits = self.split(y.into(), bits);
        self.cmp_binary(x_bits, y_bits, less, strict)
    }

    fn cmp_binary(&mut self, x_bits: BinaryExpression, y_bits: BinaryExpression,
                  less: bool, strict: bool) -> BooleanExpression {
        assert_eq!(x_bits.len(), y_bits.len());
        let operand_bits = x_bits.len();

        // We will chunk both bit vectors, then have the prover supply a mask which identifies the
        // first pair of chunks to differ. Credit to Ahmed Kosba who described this technique.
        let chunk_bits = GadgetBuilder::cmp_chunk_bits(operand_bits);
        let x_chunks: Vec<Expression> = x_bits.chunks(chunk_bits)
            .into_iter()
            .map(|c| c.join())
            .collect();
        let y_chunks: Vec<Expression> = y_bits.chunks(chunk_bits)
            .into_iter()
            .map(|c| c.join())
            .collect();
        let chunks = x_chunks.len();

        // Create a mask bit for each chunk index. masks[i] must equal 1 iff i is the first index
        // where the chunks differ, otherwise 0. If no chunks differ, all masks must equal 0.
        let mask = self.wires(chunks);
        // Each mask bit wire must equal 0 or 1.
        for &m in &mask {
            self.assert_boolean(m.into());
        }
        // The sum of all masks must equal 0 or 1, so that at most one mask can equal 1.
        let diff_exists = self.assert_boolean(Expression::sum(&mask));

        {
            let x_chunks = x_chunks.clone();
            let y_chunks = y_chunks.clone();
            let mask = mask.clone();
            self.generator(
                [x_bits.dependencies(), y_bits.dependencies()].concat(),
                move |values: &mut WireValues| {
                    let mut seen_diff: bool = false;
                    for (i, &mask_bit) in enumerate(&mask).rev() {
                        let x_chunk_value = x_chunks[i].evaluate(values);
                        let y_chunk_value = y_chunks[i].evaluate(values);
                        let diff = x_chunk_value != y_chunk_value;
                        let mask_bit_value = diff && !seen_diff;
                        seen_diff |= diff;
                        values.set(mask_bit, mask_bit_value.into());
                    }
                }
            );
        }

        // Compute the dot product of the mask vector with (x_chunks - y_chunks).
        let mut diff_chunk = Expression::zero();
        for i in 0..chunks {
            diff_chunk += self.product(mask[i].into(), x_chunks[i].clone() - y_chunks[i].clone());
        }

        // Verify that any more significant pairs of chunks are equal.
        // diff_seen tracks whether a mask bit of 1 has been observed for a less significant bit.
        let mut diff_seen: Expression = mask[0].into();
        for i in 1..chunks {
            // If diff_seen = 1, we require that x_chunk = y_chunk.
            // Equivalently, we require that diff_seen * (x_chunk - y_chunk) = 0.
            self.assert_product(diff_seen.clone(),
                                x_chunks[i].clone() - y_chunks[i].clone(),
                                0.into());
            diff_seen += mask[i].into();
        }

        // If the mask has a 1 bit, then the corresponding pair of chunks must differ. We only need
        // this check for non-strict comparisons though, since for strict comparisons, the
        // comparison operation applied to the selected chunks will enforce that they differ.
        if !strict {
            let nonzero = self._if(diff_exists,
                                   diff_chunk.clone(),
                                   // The mask is 0, so just assert that 42 (arbitrary) is non-zero.
                                   42.into());
            self.assert_nonzero(nonzero);
        }

        // Finally, apply a different comparison algorithm to the (small) differing chunks.
        self.cmp_subtractive(diff_chunk, less, strict, chunk_bits)
    }

    /// Given a diff of `x - y`, compare `x` and `y`.
    fn cmp_subtractive(&mut self, diff: Expression,
                       less: bool, strict: bool, bits: usize) -> BooleanExpression {
        // An as example, assume less=false and strict=false. In that case, we compute
        //     2^bits + x - y
        // And check the most significant bit, i.e., the one with index `bits`.
        // x >= y iff that bit is set. The other cases are similar.
        // TODO: If `bits` is very large, base might not fit in a field element. Need to generalize
        // this to work with arbitrary bit widths, or at least an assertion to fail gracefully.
        let base = Expression::from(
            (FieldElement::one() << bits) - FieldElement::from(strict));
        let z = base + if less { -diff } else { diff };
        self.split(z, bits + 1).bits[bits].clone()
    }

    /// The number of constraints used by `cmp_binary`, given a certain chunk size.
    fn cmp_constraints(operand_bits: usize, chunk_bits: usize) -> usize {
        let chunks = (operand_bits + chunk_bits - 1) / chunk_bits;
        3 * chunks + 2 + chunk_bits
    }

    /// The optimal number of bits per chunk for the comparison algorithm used in `cmp_binary`.
    fn cmp_chunk_bits(operand_bits: usize) -> usize {
        let mut best_chunk_bits = 1;
        let mut best_constraints = GadgetBuilder::cmp_constraints(operand_bits, 1);
        for chunk_bits in 2..FieldElement::max_bits() {
            let constraints = GadgetBuilder::cmp_constraints(operand_bits, chunk_bits);
            if constraints < best_constraints {
                best_chunk_bits = chunk_bits;
                best_constraints = constraints;
            }
        }
        best_chunk_bits
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use crate::field_element::FieldElement;
    use crate::gadget_builder::GadgetBuilder;
    use crate::wire_values::WireValues;
    use crate::bits::BooleanExpression;

    #[test]
    fn comparisons() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let lt = builder.lt(x.into(), y.into());
        let le = builder.le(x.into(), y.into());
        let gt = builder.gt(x.into(), y.into());
        let ge = builder.ge(x.into(), y.into());
        let gadget = builder.build();

        let mut values_42_63 = values!(x => 42.into(), y => 63.into());
        assert!(gadget.execute(&mut values_42_63));
        assert_true(&lt, &values_42_63);
        assert_true(&le, &values_42_63);
        assert_false(&gt, &values_42_63);
        assert_false(&ge, &values_42_63);

        let mut values_42_42 = values!(x => 42.into(), y => 42.into());
        assert!(gadget.execute(&mut values_42_42));
        assert_false(&lt, &values_42_42);
        assert_true(&le, &values_42_42);
        assert_false(&gt, &values_42_42);
        assert_true(&ge, &values_42_42);

        let mut values_42_41 = values!(x => 42.into(), y => 41.into());
        assert!(gadget.execute(&mut values_42_41));
        assert_false(&lt, &values_42_41);
        assert_false(&le, &values_42_41);
        assert_true(&gt, &values_42_41);
        assert_true(&ge, &values_42_41);

        // This is a white box sort of test. Since the implementation is based on chunks of roughly
        // 32 bits each, all the numbers in the preceding tests will fit into the least significant
        // chunk. So let's try some larger numbers. In particular, let's have x < y but have the
        // least significant chunk of y exceed that of x, to make sure the more significant chunk
        // takes precedence.
        let mut values_large_lt = values!(
            x => FieldElement::from(1u128 << 80 | 1u128),
            y => FieldElement::from(1u128 << 81));
        assert!(gadget.execute(&mut values_large_lt));
        assert_true(&lt, &values_large_lt);
        assert_true(&le, &values_large_lt);
        assert_false(&gt, &values_large_lt);
        assert_false(&ge, &values_large_lt);
    }

    fn assert_true<T: Borrow<BooleanExpression>>(x: T, values: &WireValues) {
        assert_eq!(true, x.borrow().evaluate(values));
    }

    fn assert_false<T: Borrow<BooleanExpression>>(x: T, values: &WireValues) {
        assert_eq!(false, x.borrow().evaluate(values));
    }
}
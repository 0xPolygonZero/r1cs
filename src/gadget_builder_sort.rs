//! This module extends GadgetBuilder with a method for splitting a field element into bits.

use itertools::enumerate;

use crate::expression::{BinaryExpression, Expression};
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;
use crate::wire::Wire;
use crate::wire_values::WireValues;

impl<F: Field> GadgetBuilder<F> {
    /// Sorts field elements in ascending order.
    pub fn sort_ascending(&mut self, inputs: &[Expression<F>]) -> Vec<Expression<F>> {
        let n = inputs.len();

        let output_wires: Vec<Wire> = self.wires(n);
        let outputs: Vec<Expression<F>> = output_wires.iter().map(Expression::from).collect();

        // First we assert that the input and output lists are permutations of one another, i.e.,
        // that they contain the same values.
        self.assert_permutation(inputs, &outputs);

        // Then, we assert the order of each adjacent pair of output values. Note that assert_le
        // would internally split each input into its binary form. To avoid splitting intermediate
        // items twice, we will explicitly split here, and call assert_le_binary instead.
        let outputs_binary: Vec<BinaryExpression<F>> = outputs.iter()
            .map(|e| self.split(e, Element::<F>::max_bits()))
            .collect();
        for i in 0..(n - 1) {
            let a = &outputs_binary[i];
            let b = &outputs_binary[i + 1];
            self.assert_le_binary(a, b);
        }

        let inputs = inputs.to_vec();
        self.generator(
            inputs.iter().flat_map(Expression::dependencies).collect(),
            move |values: &mut WireValues<F>| {
                // Evaluate all the inputs, sort that list of field elements, and output that.
                let mut items: Vec<Element<F>> =
                    inputs.iter().map(|e| e.evaluate(values)).collect();
                items.sort();
                for (i, item) in enumerate(items) {
                    values.set(output_wires[i], item);
                }
            });

        outputs
    }

    /// Sorts field elements in descending order.
    pub fn sort_descending(&mut self, inputs: &[Expression<F>]) -> Vec<Expression<F>> {
        let mut items = self.sort_ascending(inputs);
        items.reverse();
        items
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::{Bn128, Element};
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn sort_4_ascending() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let (a, b, c, d) = (builder.wire(), builder.wire(), builder.wire(), builder.wire());
        let outputs = builder.sort_ascending(&vec![
            Expression::from(a), Expression::from(b), Expression::from(c), Expression::from(d)]);
        let gadget = builder.build();

        let mut values = values!(
            a => 4u8.into(), b => 7u8.into(), c => 0u8.into(), d => 1u8.into());
        assert!(gadget.execute(&mut values));
        assert_eq!(Element::from(0u8), outputs[0].evaluate(&values));
        assert_eq!(Element::from(1u8), outputs[1].evaluate(&values));
        assert_eq!(Element::from(4u8), outputs[2].evaluate(&values));
        assert_eq!(Element::from(7u8), outputs[3].evaluate(&values));
    }

    #[test]
    fn sort_4_descending() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let (a, b, c, d) = (builder.wire(), builder.wire(), builder.wire(), builder.wire());
        let outputs = builder.sort_descending(&vec![
            Expression::from(a), Expression::from(b), Expression::from(c), Expression::from(d)]);
        let gadget = builder.build();

        let mut values = values!(
            a => 4u8.into(), b => 7u8.into(), c => 0u8.into(), d => 1u8.into());
        assert!(gadget.execute(&mut values));
        assert_eq!(Element::from(7u8), outputs[0].evaluate(&values));
        assert_eq!(Element::from(4u8), outputs[1].evaluate(&values));
        assert_eq!(Element::from(1u8), outputs[2].evaluate(&values));
        assert_eq!(Element::from(0u8), outputs[3].evaluate(&values));
    }
}
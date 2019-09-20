//! This module extends GadgetBuilder with a method for sorting lists of field elements.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use itertools::enumerate;

use crate::expression::Expression;
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
        // Also note that only the purportedly largest item (i.e. the last one) needs to be split
        // canonically. If one of the other elements were to be split into their non-canonical
        // binary encoding, that binary expression would be greater than the last element, rendering
        // the instance unsatisfiable.
        let mut outputs_binary = Vec::new();
        for out in outputs.iter().take(n - 1) {
            outputs_binary.push(self.split_allowing_ambiguity(out));
        }
        outputs_binary.push(self.split(&outputs[n - 1]));

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
                    inputs.iter().map(|exp| exp.evaluate(values)).collect();
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
    use crate::field::Element;
    use crate::gadget_builder::GadgetBuilder;
    use crate::test_util::F257;

    #[test]
    fn sort_4_ascending() {
        let mut builder = GadgetBuilder::<F257>::new();
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
        let mut builder = GadgetBuilder::<F257>::new();
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
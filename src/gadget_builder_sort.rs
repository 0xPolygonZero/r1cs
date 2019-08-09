//! This module extends GadgetBuilder with a method for splitting a field element into bits.

use std::cmp::Ordering;

use itertools::enumerate;

use crate::expression::Expression;
use crate::field_element::FieldElement;
use crate::gadget_builder::GadgetBuilder;
use crate::wire::Wire;
use crate::wire_values::WireValues;

impl GadgetBuilder {
    /// Sorts field elements in ascending order.
    pub fn sort_ascending(&mut self, inputs: &[Expression]) -> Vec<Expression> {
        let input_vec: Vec<Expression> = inputs.into_iter().map(|e| e.clone()).collect();
        let n = input_vec.len();

        let output_wires: Vec<Wire> = self.wires(n);
        let outputs: Vec<Expression> = output_wires.iter().map(Expression::from).collect();

        // First we assert that the input and output lists are permutations of one another, i.e.,
        // that they contain the same values.
        self.assert_permutation(&input_vec, &outputs);

        // Then, we assert the order of each adjacent pair of output values.
        for i in 0..(n - 1) {
            let a = &outputs[i];
            let b = &outputs[i + 1];
            self.assert_le(a, b);
        }

        self.generator(
            input_vec.iter().flat_map(Expression::dependencies).collect(),
            move |values: &mut WireValues| {
                // Evaluate all the inputs, sort that list of field elements, and output that.
                let mut items: Vec<FieldElement> =
                    input_vec.iter().map(|e| e.evaluate(values)).collect();
                items.sort();
                for (i, item) in enumerate(items) {
                    values.set(output_wires[i], item);
                }
            });

        outputs
    }

    /// Sorts field elements in descending order.
    pub fn sort_descending(&mut self, inputs: &[Expression]) -> Vec<Expression> {
        let mut items = self.sort_ascending(inputs);
        items.reverse();
        items
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field_element::FieldElement;
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn sort_4_ascending() {
        let mut builder = GadgetBuilder::new();
        let (a, b, c, d) = (builder.wire(), builder.wire(), builder.wire(), builder.wire());
        let outputs = builder.sort_ascending(&vec![
            Expression::from(a), Expression::from(b), Expression::from(c), Expression::from(d)]);
        let gadget = builder.build();

        let mut values = values!(a => 4.into(), b => 7.into(), c => 0.into(), d => 1.into());
        assert!(gadget.execute(&mut values));
        assert_eq!(FieldElement::from(0), outputs[0].evaluate(&values));
        assert_eq!(FieldElement::from(1), outputs[1].evaluate(&values));
        assert_eq!(FieldElement::from(4), outputs[2].evaluate(&values));
        assert_eq!(FieldElement::from(7), outputs[3].evaluate(&values));
    }

    #[test]
    fn sort_4_descending() {
        let mut builder = GadgetBuilder::new();
        let (a, b, c, d) = (builder.wire(), builder.wire(), builder.wire(), builder.wire());
        let outputs = builder.sort_descending(&vec![
            Expression::from(a), Expression::from(b), Expression::from(c), Expression::from(d)]);
        let gadget = builder.build();

        let mut values = values!(a => 4.into(), b => 7.into(), c => 0.into(), d => 1.into());
        assert!(gadget.execute(&mut values));
        assert_eq!(FieldElement::from(7), outputs[0].evaluate(&values));
        assert_eq!(FieldElement::from(4), outputs[1].evaluate(&values));
        assert_eq!(FieldElement::from(1), outputs[2].evaluate(&values));
        assert_eq!(FieldElement::from(0), outputs[3].evaluate(&values));
    }
}
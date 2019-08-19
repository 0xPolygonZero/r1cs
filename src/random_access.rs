//! This module extends GadgetBuilder with methods for randomly accessing lists.

use core::borrow::Borrow;

use itertools::enumerate;

use crate::expression::Expression;
use crate::field::Field;
use crate::gadget_builder::GadgetBuilder;

impl<F: Field> GadgetBuilder<F> {
    /// Access the `i`th element of `items`, where `i` may be a dynamic expression. Assumes
    /// `i < items.len()`.
    // TODO: This only supports scans for now; should support Merkle proofs for large lists.
    pub fn random_access<E>(&mut self, items: Vec<Expression<F>>, index: E) -> Expression<F>
        where E: Borrow<Expression<F>> {
        let mut result = Expression::zero();
        for (i, item) in enumerate(items) {
            let selected = self.equal(index.borrow(), Expression::from(i));
            result += self.product(selected.expression(), item);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::{Bn128, Element};
    use crate::gadget_builder::GadgetBuilder;
    use crate::wire_values::WireValues;

    #[test]
    fn random_access() {
        let n = 10;
        let mut builder = GadgetBuilder::<Bn128>::new();
        let item_wires = builder.wires(n);
        let item_exps = item_wires.iter().map(Expression::from).collect();
        let index_wire = builder.wire();
        let index_exp = Expression::from(index_wire);
        let result = builder.random_access(item_exps, index_exp);
        let gadget = builder.build();

        let mut wire_values = WireValues::new();
        for i in 0..n {
            wire_values.set(item_wires[i], Element::from(i));
        }

        for i in 0..n {
            let mut wire_values_i = wire_values.clone();
            wire_values_i.set(index_wire, Element::from(i));
            assert!(gadget.execute(&mut wire_values_i));
            assert_eq!(Element::from(i), result.evaluate(&wire_values_i));
        }
    }
}
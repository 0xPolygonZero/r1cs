//! This module extends GadgetBuilder with methods for randomly accessing lists.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::BooleanExpression;
use crate::expression::Expression;
use crate::field::Field;
use crate::gadget_builder::GadgetBuilder;

impl<F: Field> GadgetBuilder<F> {
    /// Access the `i`th element of `items`, where `i` may be a dynamic expression. Assumes
    /// `i < items.len()`.
    ///
    /// Note that this gadget returns 0 if the index is out of range. If you want to prohibit
    /// out-of-range indices, you can do so with a separate call to `assert_lt`.
    pub fn random_access(
        &mut self,
        items: &[Expression<F>],
        index: &Expression<F>,
    ) -> Expression<F> {
        // Determine the minimum number of bits needed to encode the index.
        let mut bits = 0;
        while 1 << bits < items.len() {
            bits += 1;
        }

        let index_binary = self.split_bounded(index, bits);
        self.random_access_binary(items, index_binary.bits)
    }

    /// Like `random_access`, but with a binary index.
    fn random_access_binary(
        &mut self,
        items: &[Expression<F>],
        mut index_bits: Vec<BooleanExpression<F>>,
    ) -> Expression<F> {
        // Imagine a perfect binary tree whose leaves consist of the given items, followed by zeros
        // for padding. We can think of each bit of the index as filtering a single layer of the
        // tree. For example, the first (least significant) index bit selects between pairs of
        // leaves. After filtering each layer in this manner, we are left with a single value
        // corresponding to the root of the tree.

        // This leads to a natural recursive solution. Each call of this method will filter the
        // deepest layer of the tree, then recurse, until we are left with a singleton list.

        if items.len() == 1 {
            assert!(index_bits.is_empty());
            return items[0].clone();
        }

        let lsb = index_bits.remove(0);
        let num_parents = (items.len() + 1) / 2;
        let mut parent_layer = Vec::with_capacity(num_parents);
        for parent_index in 0..num_parents {
            let left_child_index = parent_index * 2;
            let right_child_index = parent_index * 2 + 1;
            let left_child = &items[left_child_index];
            if right_child_index == items.len() {
                parent_layer.push(left_child.clone());
            } else {
                let right_child = &items[right_child_index];
                parent_layer.push(self.selection(&lsb, right_child, left_child));
            }
        }

        self.random_access_binary(&parent_layer, index_bits)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::expression::Expression;
    use crate::field::Element;
    use crate::gadget_builder::GadgetBuilder;
    use crate::test_util::F257;
    use crate::wire_values::WireValues;

    #[test]
    fn random_access() {
        let n = 10;
        let mut builder = GadgetBuilder::<F257>::new();
        let item_wires = builder.wires(n);
        let item_exps = item_wires.iter().map(Expression::from).collect_vec();
        let index_wire = builder.wire();
        let index_exp = Expression::from(index_wire);
        let result = builder.random_access(&item_exps, &index_exp);
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
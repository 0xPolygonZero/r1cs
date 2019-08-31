use crate::expression::{BinaryExpression, BooleanExpression, Expression};
use crate::field::Field;
use crate::gadget_builder::GadgetBuilder;
use crate::gadget_traits::CompressionFunction;

/// The path from a leaf to the root of a binary Merkle tree.
#[derive(Debug)]
pub struct MerklePath<F: Field> {
    /// The sequence of "turns" when traversing up the tree. The value of each bit indicates the
    /// index of the target node relative to its parent. For example, a zero bit indicates that the
    /// target node is the left child, and its sibling is the right child.
    prefix: BinaryExpression<F>,
    /// The sequence of (hashes of) sibling nodes which are encountered along the path up the tree.
    siblings: Vec<Expression<F>>,
}

impl<F: Field> MerklePath<F> {
    pub fn new(prefix: BinaryExpression<F>, siblings: Vec<Expression<F>>) -> Self {
        assert_eq!(prefix.len(), siblings.len());
        MerklePath { prefix, siblings }
    }
}

impl<F: Field> Clone for MerklePath<F> {
    fn clone(&self) -> Self {
        MerklePath {
            prefix: self.prefix.clone(),
            siblings: self.siblings.clone(),
        }
    }
}

impl<F: Field> GadgetBuilder<F> {
    /// Update an intermediate hash value in a Merkle tree, given the sibling at the current layer.
    fn merkle_tree_step<CF>(
        &mut self,
        node: &Expression<F>,
        sibling: &Expression<F>,
        prefix_bit: &BooleanExpression<F>,
        compress: &CF,
    ) -> Expression<F> where CF: CompressionFunction<F> {
        let left = self.selection(prefix_bit, sibling, node);
        let right = sibling + node - &left;
        compress.compress(self, &left, &right)
    }

    /// Compute a Merkle root given a leaf value and its Merkle path.
    pub fn merkle_tree_root<CF>(
        &mut self,
        leaf: &Expression<F>,
        path: &MerklePath<F>,
        compress: &CF,
    ) -> Expression<F> where CF: CompressionFunction<F> {
        let mut current = leaf.clone();
        for (prefix_bit, sibling) in path.prefix.bits.iter().zip(path.siblings.iter()) {
            current = self.merkle_tree_step(
                &current, sibling, prefix_bit, compress);
        }
        current
    }

    pub fn assert_merkle_tree_membership<E1, E2, MP, CF>(
        &mut self,
        leaf: &Expression<F>,
        purported_root: &Expression<F>,
        path: &MerklePath<F>,
        compress: &CF,
    ) where CF: CompressionFunction<F> {
        let computed_root = self.merkle_tree_root(leaf, path, compress);
        self.assert_equal(purported_root, &computed_root)
    }
}

#[cfg(test)]
mod tests {
    use num::BigUint;

    use crate::expression::{BinaryExpression, BooleanExpression, Expression};
    use crate::field::{Element, Field};
    use crate::gadget_builder::GadgetBuilder;
    use crate::gadget_traits::CompressionFunction;
    use crate::merkle_trees::MerklePath;
    use crate::test_util::F257;

    #[test]
    fn merkle_step() {
        let mut builder = GadgetBuilder::<F257>::new();
        let node = builder.wire();
        let sibling = builder.wire();
        let is_right = builder.boolean_wire();
        let parent_hash = builder.merkle_tree_step(
            &Expression::from(node), &Expression::from(sibling),
            &BooleanExpression::from(is_right), &TestCompress);
        let gadget = builder.build();

        let mut values_3_4 = values!(node => 3u8.into(), sibling => 4u8.into());
        values_3_4.set_boolean(is_right, false);
        assert!(gadget.execute(&mut values_3_4));
        assert_eq!(Element::from(10u8), parent_hash.evaluate(&values_3_4));

        let mut values_4_3 = values!(node => 3u8.into(), sibling => 4u8.into());
        values_4_3.set_boolean(is_right, true);
        assert!(gadget.execute(&mut values_4_3));
        assert_eq!(Element::from(11u8), parent_hash.evaluate(&values_4_3));
    }

    #[test]
    fn merkle_root() {
        let mut builder = GadgetBuilder::<F257>::new();
        let prefix_wire = builder.binary_wire(3);
        let (sibling_1, sibling_2, sibling_3) = (builder.wire(), builder.wire(), builder.wire());
        let path = MerklePath::new(
            BinaryExpression::from(&prefix_wire),
            vec![sibling_1.into(), sibling_2.into(), sibling_3.into()]);
        let root_hash = builder.merkle_tree_root(&Expression::one(), &path, &TestCompress);
        let gadget = builder.build();

        let mut values = values!(
            sibling_1 => 3u8.into(),
            sibling_2 => 3u8.into(),
            sibling_3 => 9u8.into());
        values.set_binary_unsigned(&prefix_wire, &BigUint::from(0b010u8));
        assert!(gadget.execute(&mut values));
        // The leaf is 1; the first parent hash is 2*1 + 3 = 5; the next parent hash is
        // 2*3 + 5 = 11; the root is 2*11 + 9 = 31.
        assert_eq!(Element::from(31u8), root_hash.evaluate(&values));
    }

    // A dummy compression function which returns 2x + y.
    struct TestCompress;

    impl<F: Field> CompressionFunction<F> for TestCompress {
        fn compress(&self, _builder: &mut GadgetBuilder<F>, x: &Expression<F>, y: &Expression<F>)
                    -> Expression<F> {
            x * 2 + y
        }
    }
}
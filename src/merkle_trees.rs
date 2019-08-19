use crate::expression::{BinaryExpression, Expression, BooleanExpression};
use crate::field::Field;
use crate::gadget_builder::GadgetBuilder;
use std::borrow::Borrow;

type CompressionFunction<F> = fn(&mut GadgetBuilder<F>, Expression<F>, Expression<F>)
                                 -> Expression<F>;

/// The path from a leaf to the root of a binary Merkle tree.
#[derive(Clone, Debug)]
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

impl<F: Field> GadgetBuilder<F> {
    /// Update an intermediate hash value in a Merkle tree, given the sibling at the current layer.
    fn merkle_tree_step<E1, E2, BE>(
        &mut self,
        node: E1,
        sibling: E2,
        prefix_bit: BE,
        compress: CompressionFunction<F>,
    ) -> Expression<F>
        where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>>,
              BE: Borrow<BooleanExpression<F>> {
        let node = node.borrow();
        let sibling = sibling.borrow();
        let left = self.selection(prefix_bit, sibling, node);
        let right = sibling + node - &left;
        compress(self, left, right)
    }

    /// Compute a Merkle root given a leaf value and its Merkle path.
    fn merkle_tree_root<E, MP>(
        &mut self,
        leaf: E,
        path: MP,
        compress: CompressionFunction<F>,
    ) -> Expression<F> where E: Borrow<Expression<F>>, MP: Borrow<MerklePath<F>> {
        let path = path.borrow();
        let mut current = leaf.borrow().clone();
        for (prefix_bit, sibling) in path.prefix.bits.iter().zip(path.siblings.iter()) {
            current = self.merkle_tree_step(current, sibling.clone(), prefix_bit.clone(), compress);
        }
        current
    }

    pub fn assert_merkle_tree_membership<E1, E2, MP>(
        &mut self,
        leaf: E1,
        purported_root: E2,
        path: MerklePath<F>,
        compress: CompressionFunction<F>,
    ) where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>>, MP: Borrow<MerklePath<F>> {
        let computed_root = self.merkle_tree_root(leaf, path, compress);
        self.assert_equal(purported_root, computed_root)
    }
}
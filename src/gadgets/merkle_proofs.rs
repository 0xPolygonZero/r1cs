use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;
use wire::Wire;

type CompressionFunction = fn(&mut GadgetBuilder, LinearCombination, LinearCombination)
                              -> LinearCombination;

/// The path from a leaf to the root of a binary Merkle tree.
#[derive(Clone)]
pub struct MerklePath {
    prefix: Vec<Wire>,
    siblings: Vec<LinearCombination>,
}

impl MerklePath {
    pub fn new(prefix: Vec<Wire>, siblings: Vec<LinearCombination>) -> Self {
        assert_eq!(prefix.len(), siblings.len());
        MerklePath { prefix, siblings }
    }
}

impl GadgetBuilder {
    /// Update an intermediate hash value in a Merkle trie, given the sibling at the current layer.
    fn merkle_trie_step(&mut self, node: LinearCombination, sibling: LinearCombination,
                        prefix_bit: Wire, compress: CompressionFunction) -> LinearCombination {
        let node_is_right = prefix_bit;
        let node_is_left = LinearCombination::one() - LinearCombination::from(node_is_right);
        let left = self.product(node_is_left.clone(), node.clone())
            + self.product(node_is_right.into(), sibling.clone());
        let right = self.product(node_is_right.into(), node)
            + self.product(node_is_left, sibling);
        compress(self, left, right)
    }

    /// Compute the Merkle root given a leaf value (0 or 1), a prefix and each sibling. Each wire in
    /// the prefix list must be 0 or 1; it is assumed that this is enforced elsewhere, e.g. by a
    /// split gate. The prefix list and the sibling list are ordered from the bottom of the tree to
    /// the top.
    fn merkle_trie_root(&mut self, leaf: bool, path: MerklePath, compress: CompressionFunction)
                        -> LinearCombination {
        let mut current = if leaf { LinearCombination::one() } else { LinearCombination::zero() };
        for (prefix_bit, sibling) in path.prefix.iter().zip(path.siblings.iter()) {
            current = self.merkle_trie_step(current, sibling.clone(), prefix_bit.clone(), compress);
        }
        current
    }

    /// Assert that a given prefix is present in the trie with the given root.
    pub fn merkle_trie_assert_membership(&mut self, path: MerklePath, root: LinearCombination,
                                         compress: CompressionFunction) {
        let root_with_prefix = self.merkle_trie_root(true, path, compress);
        self.assert_equal(root_with_prefix, root);
    }

    /// Assert that a given prefix is not present in the trie with the given root.
    pub fn merkle_trie_assert_nonmembership(&mut self, path: MerklePath, root: LinearCombination,
                                            compress: CompressionFunction) {
        let root_without_prefix = self.merkle_trie_root(false, path, compress);
        self.assert_equal(root_without_prefix, root);
    }

    /// Compute the Merkle roots before and after a prefix was inserted.
    pub fn merkle_trie_insert(&mut self, path: MerklePath, compress: CompressionFunction)
                              -> (LinearCombination, LinearCombination) {
        let root_without_prefix = self.merkle_trie_root(false, path.clone(), compress);
        let root_with_prefix = self.merkle_trie_root(true, path, compress);
        (root_without_prefix, root_with_prefix)
    }

    /// Compute the Merkle roots before and after a prefix was deleted.
    pub fn merkle_trie_delete(&mut self, path: MerklePath, compress: CompressionFunction)
                              -> (LinearCombination, LinearCombination) {
        let mut root_without_prefix = self.merkle_trie_root(false, path.clone(), compress);
        let mut root_with_prefix = self.merkle_trie_root(true, path, compress);
        (root_with_prefix, root_without_prefix)
    }
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;
    use linear_combination::LinearCombination;
    use field_element::FieldElement;
    use gadgets::merkle_proofs::MerklePath;

    #[test]
    fn mimc_merkle_step() {
        let mut builder = GadgetBuilder::new();
        let (node, sibling, is_right) = (builder.wire(), builder.wire(), builder.wire());
        let parent_hash = builder.merkle_trie_step(
            node.into(), sibling.into(), is_right, test_compress);
        let gadget = builder.build();

        let mut values_3_4 = wire_values!(
            node => 3.into(),
            sibling => 4.into(),
            is_right => 0.into());
        assert!(gadget.execute(&mut values_3_4));
        assert_eq!(FieldElement::from(10), parent_hash.evaluate(&values_3_4));

        let mut values_4_3 = wire_values!(
            node => 3.into(),
            sibling => 4.into(),
            is_right => 1.into());
        assert!(gadget.execute(&mut values_4_3));
        assert_eq!(FieldElement::from(11), parent_hash.evaluate(&values_4_3));
    }

    #[test]
    fn mimc_merkle_root() {
        let mut builder = GadgetBuilder::new();
        let (sibling_1, is_right_1) = (builder.wire(), builder.wire());
        let (sibling_2, is_right_2) = (builder.wire(), builder.wire());
        let (sibling_3, is_right_3) = (builder.wire(), builder.wire());
        let path = MerklePath::new(
            vec![is_right_1, is_right_2, is_right_3],
            vec![sibling_1.into(), sibling_2.into(), sibling_3.into()]);
        let root_hash = builder.merkle_trie_root(true, path, test_compress);
        let gadget = builder.build();

        let mut values = wire_values!(
            sibling_1 => 3.into(),
            is_right_1 => 0.into(),
            sibling_2 => 3.into(),
            is_right_2 => 1.into(),
            sibling_3 => 9.into(),
            is_right_3 => 0.into());
        assert!(gadget.execute(&mut values));
        // The leaf is 1; the first parent hash is 2*1 + 3 = 5; the next parent hash is
        // 2*3 + 5 = 11; the root is 2*11 + 9 = 31.
        assert_eq!(FieldElement::from(31), root_hash.evaluate(&values));
    }

    // A dummy compression function which returns 2x + y.
    fn test_compress(_builder: &mut GadgetBuilder, x: LinearCombination, y: LinearCombination)
                     -> LinearCombination {
        x * 2 + y
    }
}
use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;

type CompressionFunction = fn(&mut GadgetBuilder, LinearCombination, LinearCombination)
                              -> LinearCombination;

impl GadgetBuilder {
    // TODO: Take a Lemma parameter instead? Since this will likely only be called from merkle_root.
    fn merkle_step(&mut self, subject_hash: LinearCombination, sibling_hash: LinearCombination,
                   subject_is_left: LinearCombination, hash_function: CompressionFunction,
    ) -> LinearCombination {
        self.assert_binary(subject_is_left.clone());
        let subject_is_right = LinearCombination::one() - subject_is_left.clone();
        let left = self.product(subject_is_left.clone(), subject_hash.clone())
            + self.product(subject_is_right.clone(), sibling_hash.clone());
        let right = self.product(subject_is_right, subject_hash)
            + self.product(subject_is_left, sibling_hash);
        hash_function(self, left, right)
    }

    /// Verify a membership proof for any binary Merkle tree.
    pub fn merkle_root<'a, T>(&mut self, leaf_hash: LinearCombination, proof: T,
                              hash_function: CompressionFunction) -> LinearCombination
        where T: IntoIterator<Item=&'a Lemma> {
        let mut current = leaf_hash;
        for lemma in proof {
            current = self.merkle_step(current, lemma.sibling_hash.clone(),
                                       lemma.subject_is_left.clone(), hash_function)
        }
        current
    }
}

/// A piece of the Merkle proof corresponding to a single layer of the tree.
pub struct Lemma {
    subject_is_left: LinearCombination,
    sibling_hash: LinearCombination,
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;
    use linear_combination::LinearCombination;
    use field_element::FieldElement;
    use gadgets::merkle::Lemma;

    #[test]
    fn mimc_merkle_step() {
        let mut builder = GadgetBuilder::new();
        let (subject, sibling, is_left) = (builder.wire(), builder.wire(), builder.wire());
        let parent_hash = builder.merkle_step(subject.into(), sibling.into(), is_left.into(), test_hash);
        let gadget = builder.build();

        let mut values_3_4 = wire_values!(
            subject => 3.into(),
            sibling => 4.into(),
            is_left => 1.into());
        assert!(gadget.execute(&mut values_3_4));
        assert_eq!(FieldElement::from(10), parent_hash.evaluate(&values_3_4));

        let mut values_4_3 = wire_values!(
            subject => 3.into(),
            sibling => 4.into(),
            is_left => 0.into());
        assert!(gadget.execute(&mut values_4_3));
        assert_eq!(FieldElement::from(11), parent_hash.evaluate(&values_4_3));
    }

    #[test]
    fn mimc_merkle_root() {
        let mut builder = GadgetBuilder::new();
        let leaf = builder.wire();
        let (sibling_1, is_left_1) = (builder.wire(), builder.wire());
        let (sibling_2, is_left_2) = (builder.wire(), builder.wire());
        let (sibling_3, is_left_3) = (builder.wire(), builder.wire());
        let lemmas = vec![
            Lemma { subject_is_left: is_left_1.into(), sibling_hash: sibling_1.into() },
            Lemma { subject_is_left: is_left_2.into(), sibling_hash: sibling_2.into() },
            Lemma { subject_is_left: is_left_3.into(), sibling_hash: sibling_3.into() },
        ];
        let root_hash = builder.merkle_root(leaf.into(), &lemmas, test_hash);
        let gadget = builder.build();

        let mut values = wire_values!(
            leaf => 1.into(),
            sibling_1 => 3.into(),
            is_left_1 => 1.into(),
            sibling_2 => 3.into(),
            is_left_2 => 0.into(),
            sibling_3 => 9.into(),
            is_left_3 => 1.into());
        assert!(gadget.execute(&mut values));
        // The leaf is 1; the first parent hash is 2*1 + 3 = 5; the next parent hash is
        // 2*3 + 5 = 11; the root is 2*11 + 9 = 31.
        assert_eq!(FieldElement::from(31), root_hash.evaluate(&values));
    }

    // A dummy hash function which returns 2x + y.
    fn test_hash(_builder: &mut GadgetBuilder, x: LinearCombination, y: LinearCombination)
                 -> LinearCombination {
        x * 2 + y
    }
}
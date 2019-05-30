use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;

type HashFunction = fn(&mut GadgetBuilder, LinearCombination, LinearCombination) -> LinearCombination;

impl GadgetBuilder {
    pub fn merkle_step(&mut self, subject_hash: LinearCombination, sibling_hash: LinearCombination,
                       subject_is_left: LinearCombination, hash_function: HashFunction,
    ) -> LinearCombination {
        let subject_is_right = LinearCombination::one() - subject_is_left.clone();
        let left = self.product(subject_is_left.clone(), subject_hash.clone())
            + self.product(subject_is_right.clone(), sibling_hash.clone());
        let right = self.product(subject_is_right, subject_hash)
            + self.product(subject_is_left, sibling_hash);
        hash_function(self, left, right)
    }

    /// Verify a membership proof for any binary Merkle tree.
    pub fn merkle_root<'a, T>(leaf_hash: LinearCombination, proof: T, hash_function: HashFunction)
        where T: IntoIterator<Item=&'a Lemma> {
        unimplemented!("TODO")
    }
}

/// A piece of the Merkle proof corresponding to a single layer of the tree.
pub struct Lemma {
    is_left: LinearCombination,
    sibling_hash: LinearCombination,
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;
    use linear_combination::LinearCombination;
    use field_element::FieldElement;

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

    // A dummy hash function which returns 2x + y.
    fn test_hash(builder: &mut GadgetBuilder, x: LinearCombination, y: LinearCombination)
                 -> LinearCombination {
        x * 2 + y
    }
}
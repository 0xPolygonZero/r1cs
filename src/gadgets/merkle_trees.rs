use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;

type CompressionFunction = fn(&mut GadgetBuilder, LinearCombination, LinearCombination)
                              -> LinearCombination;

pub struct MembershipProof {
    lemmas: Vec<MembershipLemma>,
}

/// A piece of the Merkle proof corresponding to a single layer of the tree.
pub struct MembershipLemma {
    subject_is_right: LinearCombination,
    sibling: LinearCombination,
}

pub struct TrieInsertionProof {
    lemmas: Vec<TrieInsertionLemma>,
}

pub struct TrieInsertionLemma {
    subject_is_right: LinearCombination,
    old_sibling: LinearCombination,
    new_sibling: LinearCombination,
}

pub struct TrieDeletionProof {
    lemmas: Vec<TrieDeletionLemma>,
}

pub struct TrieDeletionLemma {
    subject_is_right: LinearCombination,
    old_sibling: LinearCombination,
    new_sibling: LinearCombination,
}

impl GadgetBuilder {
    fn merkle_step(&mut self, subject: LinearCombination, lemma: MembershipLemma,
                   compress: CompressionFunction) -> LinearCombination {
        self.assert_binary(lemma.subject_is_right.clone());
        let subject_is_left = LinearCombination::one() - lemma.subject_is_right.clone();
        let left = self.product(subject_is_left.clone(), subject.clone())
            + self.product(lemma.subject_is_right.clone(), lemma.sibling.clone());
        let right = self.product(lemma.subject_is_right, subject)
            + self.product(subject_is_left, lemma.sibling);
        compress(self, left, right)
    }

    /// Verify a membership proof for any binary Merkle tree.
    pub fn merkle_root(&mut self, leaf: LinearCombination, proof: MembershipProof,
                       compress: CompressionFunction) -> LinearCombination {
        let mut current = leaf;
        for lemma in proof.lemmas {
            current = self.merkle_step(current, lemma, compress)
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;
    use linear_combination::LinearCombination;
    use field_element::FieldElement;
    use gadgets::merkle_trees::{MembershipLemma, MembershipProof};

    #[test]
    fn mimc_merkle_step() {
        let mut builder = GadgetBuilder::new();
        let (subject, sibling, is_right) = (builder.wire(), builder.wire(), builder.wire());
        let lemma = MembershipLemma { subject_is_right: is_right.into(), sibling: sibling.into() };
        let parent_hash = builder.merkle_step(subject.into(), lemma, test_compress);
        let gadget = builder.build();

        let mut values_3_4 = wire_values!(
            subject => 3.into(),
            sibling => 4.into(),
            is_right => 0.into());
        assert!(gadget.execute(&mut values_3_4));
        assert_eq!(FieldElement::from(10), parent_hash.evaluate(&values_3_4));

        let mut values_4_3 = wire_values!(
            subject => 3.into(),
            sibling => 4.into(),
            is_right => 1.into());
        assert!(gadget.execute(&mut values_4_3));
        assert_eq!(FieldElement::from(11), parent_hash.evaluate(&values_4_3));
    }

    #[test]
    fn mimc_merkle_root() {
        let mut builder = GadgetBuilder::new();
        let leaf = builder.wire();
        let (sibling_1, is_right_1) = (builder.wire(), builder.wire());
        let (sibling_2, is_right_2) = (builder.wire(), builder.wire());
        let (sibling_3, is_right_3) = (builder.wire(), builder.wire());
        let lemmas = vec![
            MembershipLemma { subject_is_right: is_right_1.into(), sibling: sibling_1.into() },
            MembershipLemma { subject_is_right: is_right_2.into(), sibling: sibling_2.into() },
            MembershipLemma { subject_is_right: is_right_3.into(), sibling: sibling_3.into() },
        ];
        let proof = MembershipProof { lemmas };
        let root_hash = builder.merkle_root(leaf.into(), proof, test_compress);
        let gadget = builder.build();

        let mut values = wire_values!(
            leaf => 1.into(),
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
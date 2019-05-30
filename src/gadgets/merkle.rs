use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;

impl GadgetBuilder {
    pub fn merkle_step(&mut self, subject_hash: LinearCombination, sibling_hash: LinearCombination,
                       subject_is_left: LinearCombination,
                       hash: fn(&mut GadgetBuilder, LinearCombination, LinearCombination) -> LinearCombination
    ) -> LinearCombination {
        let subject_is_right = LinearCombination::one() - subject_is_left.clone();
        let left = self.product(subject_is_left.clone(), subject_hash.clone())
            + self.product(subject_is_right.clone(), sibling_hash.clone());
        let right = self.product(subject_is_right, subject_hash)
            + self.product(subject_is_left, sibling_hash);
        hash(self, left, right)
    }

    /// Verify a membership proof for any binary Merkle tree.
    pub fn compute_root<'a, T>(proof: T)
        where T: IntoIterator<Item=&'a Lemma> {
        unimplemented!("TODO")
    }
}

pub struct Lemma {
    is_left: LinearCombination,
    sibling_hash: LinearCombination,
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;

    #[test]
    fn mimc_merkle_step() {
        let mut builder = GadgetBuilder::new();
        let (subject, sibling, is_left) = (builder.wire(), builder.wire(), builder.wire());
        builder.merkle_step(subject.into(), sibling.into(), is_left.into(),
                            GadgetBuilder::mimc_dm_hash);
        let gadget = builder.build();

        let mut values = wire_values!(
            subject => 123.into(),
            sibling => 456.into(),
            is_left => 0.into());
        assert!(gadget.execute(&mut values));
    }
}
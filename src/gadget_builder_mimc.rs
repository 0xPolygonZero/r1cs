//! This module extends GadgetBuilder with an implementation of MiMC.

use std::borrow::Borrow;

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

use crate::expression::Expression;
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;

impl<F: Field> GadgetBuilder<F> {
    /// The MiMC block cipher in its more raw form. This method takes a list of round constants as
    /// input. The number of rounds will be one greater than the length of that list (since the
    /// first round has no random constant).
    pub fn mimc<E1, E2>(&mut self, key: E1, input: E2, round_constants: &[Element<F>])
                        -> Expression<F>
        where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>> {
        assert!(Element::<F>::largest_element().integer_modulus(Element::from(3u8)).is_nonzero(),
                "MiMC requires a field with gcd(3, p − 1) = 1");

        let key = key.borrow();
        let input = input.borrow();
        let mut current = input.clone();

        // In the first round, there is no round constant, so just add the key.
        current += key;

        // Cube the current value.
        current = self.exp(current, 3);

        for round_constant in round_constants {
            // Add the key and the random round constant.
            current += key + Expression::from(round_constant);

            // Cube the current value.
            current = self.exp(current, 3);
        }

        // Final key addition, as per the spec.
        current + key
    }

    /// The MiMC block cipher, using the recommended number of rounds, and using ChaCha20 (seeded
    /// with 0) as the source of randomness for round constants.
    pub fn mimc_chacha20<E1, E2>(&mut self, key: E1, input: E2) -> Expression<F>
        where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>> {
        self.mimc_chacha20_refs(key.borrow(), input.borrow())
    }

    /// Like `mimc_chacha20`, but takes plain references instead of `Borrow`s.
    fn mimc_chacha20_refs(&mut self, key: &Expression<F>, input: &Expression<F>) -> Expression<F> {
        let mut rng = ChaChaRng::seed_from_u64(0);
        let mut round_constants = Vec::new();
        for _r in 0..Self::mimc_recommended_rounds() {
            round_constants.push(Element::random(&mut rng));
        }
        self.mimc(key, input, &round_constants)
    }

    /// A compression function based on MiMC and the additive variant of Davies-Meyer. Uses ChaCha20
    /// (seeded with 0) as the source of randomness for MiMC round constants.
    pub fn mimc_compress<E1, E2>(&mut self, x: E1, y: E2) -> Expression<F>
        where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>> {
        self.mimc_compress_refs(x.borrow(), y.borrow())
    }

    /// Like `mimc_compress`, but takes plain references instead of `Borrow`s.
    pub fn mimc_compress_refs(&mut self, x: &Expression<F>, y: &Expression<F>) -> Expression<F> {
        self.davies_meyer(x, y, Self::mimc_chacha20_refs)
    }

    /// A hash function based on MiMC, the additive variant of Davies-Meyer, and Merkle-Damgard.
    /// Uses ChaCha20 (seeded with 0) as the source of randomness for constants.
    pub fn mimc_hash(&mut self, blocks: &[Expression<F>]) -> Expression<F> {
        self.merkle_damgard_chacha20(blocks, Self::mimc_compress_refs)
    }

    /// The recommended number of rounds to use in MiMC, based on the paper.
    pub fn mimc_recommended_rounds() -> usize {
        let n = Element::<F>::max_bits();
        (n as f64 / 3f64.log2()).ceil() as usize
    }
}

#[cfg(test)]
mod tests {
    use num::BigUint;

    use crate::expression::Expression;
    use crate::field::{Element, Field};
    use crate::gadget_builder::GadgetBuilder;
    use crate::test_util::{F11, F7};

    #[test]
    fn mimc_f11() {
        let constants = &[Element::from(5u8), Element::from(7u8)];

        let mut builder = GadgetBuilder::<F11>::new();
        let key_wire = builder.wire();
        let input_wire = builder.wire();
        let key = Expression::from(key_wire);
        let input = Expression::from(input_wire);
        let mimc = builder.mimc(key, input, constants);
        let gadget = builder.build();

        let mut values = values!(key_wire => 3u8.into(), input_wire => 6u8.into());
        assert!(gadget.execute(&mut values));
        assert_eq!(Element::from(2u8), mimc.evaluate(&values));
    }

    /// MiMC is incompatible with F_7, because cubing is not a permutation in this field.
    #[test]
    #[should_panic]
    fn mimc_f7_incompatible() {
        let mut builder = GadgetBuilder::<F7>::new();
        builder.mimc_chacha20(Expression::zero(), Expression::zero());
    }
}
//! This module extends GadgetBuilder with an implementation of MiMC.

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

use crate::expression::Expression;
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;
use crate::gadget_traits::{BlockCipher, Permutation};
use crate::MonomialPermutation;

/// The MiMC block cipher.
pub struct MiMCBlockCipher<F: Field> {
    round_constants: Vec<Element<F>>,
    round_permutation: MonomialPermutation<F>,
}

impl<F: Field> MiMCBlockCipher<F> {
    /// Creates an instance of the MiMC block cipher with the given round constants, which should be
    /// generated randomly.
    ///
    /// The number of rounds will be `round_constants.len() + 1`, since the first round has no
    /// random constant.
    fn new(round_constants: &[Element<F>]) -> Self {
        let round_permutation = MonomialPermutation::new(Element::from(3u8));
        let round_constants = round_constants.to_vec();
        MiMCBlockCipher { round_permutation, round_constants }
    }
}

impl<F: Field> Default for MiMCBlockCipher<F> {
    /// Configures MiMC with the number of rounds recommended in the paper. Uses ChaCha20 (seeded
    /// with 0) as the source of randomness for these constants.
    fn default() -> Self {
        let mut rng = ChaChaRng::seed_from_u64(0);
        let mut round_constants = Vec::new();
        for _r in 0..mimc_recommended_rounds::<F>() {
            round_constants.push(Element::random(&mut rng));
        }
        MiMCBlockCipher::new(&round_constants)
    }
}

impl<F: Field> BlockCipher<F> for MiMCBlockCipher<F> {
    fn encrypt(&self, builder: &mut GadgetBuilder<F>, key: &Expression<F>, input: &Expression<F>)
               -> Expression<F> {
        let mut current = input.clone();

        // In the first round, there is no round constant, so just add the key.
        current += key;

        // Cube the current value.
        current = self.round_permutation.permute(builder, &current);

        for round_constant in self.round_constants.iter() {
            // Add the key and the random round constant.
            current += key + Expression::from(round_constant);

            // Cube the current value.
            current = self.round_permutation.permute(builder, &current);
        }

        // Final key addition, as per the spec.
        current + key
    }

    fn decrypt(&self, builder: &mut GadgetBuilder<F>, key: &Expression<F>, output: &Expression<F>)
               -> Expression<F> {
        let mut current = output.clone();

        // Undo final key adddition.
        current -= key;

        for round_constant in self.round_constants.iter().rev() {
            // Undo the cubing permutation.
            current = self.round_permutation.inverse(builder, &current);

            // Undo the key and random round constant additions.
            current -= key + Expression::from(round_constant);
        }

        // Undo the first round cubing and key addition. (There is no constant in the first round.)
        current = self.round_permutation.inverse(builder, &current);
        current - key
    }
}

/// The MiMC permutation, which is equivalent to MiMC encryption with a key of zero.
// TODO: Consider merging the two structs.
// TODO: Implement Default.
pub struct MiMCPermutation<F: Field> {
    cipher: MiMCBlockCipher<F>
}

impl<F: Field> Permutation<F> for MiMCPermutation<F> {
    fn permute(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        // As per the paper, we just use a key of zero.
        self.cipher.encrypt(builder, &Expression::zero(), x)
    }

    fn inverse(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        self.cipher.decrypt(builder, &Expression::zero(), x)
    }
}

/// The recommended number of rounds to use in MiMC, based on the paper.
fn mimc_recommended_rounds<F: Field>() -> usize {
    let n = Element::<F>::max_bits();
    (n as f64 / 3f64.log2()).ceil() as usize
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::Element;
    use crate::gadget_builder::GadgetBuilder;
    use crate::gadget_traits::BlockCipher;
    use crate::mimc::MiMCBlockCipher;
    use crate::test_util::{F11, F7};

    #[test]
    fn mimc_encrypt_and_decrypt() {
        let mut builder = GadgetBuilder::<F11>::new();
        let key_wire = builder.wire();
        let input_wire = builder.wire();
        let key = Expression::from(key_wire);
        let input = Expression::from(input_wire);
        let mimc = MiMCBlockCipher::default();
        let encrypted = mimc.encrypt(&mut builder, &key, &input);
        let decrypted = mimc.decrypt(&mut builder, &key, &encrypted);
        let gadget = builder.build();

        let mut values = values!(key_wire => 2u8.into(), input_wire => 3u8.into());
        assert!(gadget.execute(&mut values));
        assert_eq!(input.evaluate(&values), decrypted.evaluate(&values));
    }

    #[test]
    fn mimc_f11() {
        let constants = &[Element::from(5u8), Element::from(7u8)];

        let mut builder = GadgetBuilder::<F11>::new();
        let key_wire = builder.wire();
        let input_wire = builder.wire();
        let key = Expression::from(key_wire);
        let input = Expression::from(input_wire);
        let mimc = MiMCBlockCipher::new(constants);
        let mimc_output = mimc.encrypt(&mut builder, &key, &input);
        let gadget = builder.build();

        let mut values = values!(key_wire => 3u8.into(), input_wire => 6u8.into());
        assert!(gadget.execute(&mut values));
        assert_eq!(Element::from(2u8), mimc_output.evaluate(&values));
    }

    /// MiMC is incompatible with F_7, because cubing is not a permutation in this field.
    #[test]
    #[should_panic]
    fn mimc_f7_incompatible() {
        MiMCBlockCipher::<F7>::default();
    }
}
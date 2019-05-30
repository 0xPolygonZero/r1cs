use field_element::FieldElement;
use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;

impl GadgetBuilder {
    /// The MiMC block cipher.
    pub fn mimc(&mut self, key: LinearCombination, input: LinearCombination)
                -> LinearCombination {
        // Our source of randomness for the round constants will be ChaCha20 with a seed of 0.
        let mut rng = ChaChaRng::seed_from_u64(0);
        let mut current = input;

        for r in 0..num_rounds() {
            // As per the spec, we don't add a random constant in round 0.
            let round_constant = if r == 0 {
                FieldElement::zero()
            } else {
                FieldElement::random(&mut rng)
            };

            // Add the key and the random round constant to the current value.
            current += key.clone() + round_constant.into();

            // Cube the current value.
            current = self.exp(current, 3);
        }

        // Final key addition, as per the spec.
        current + key.clone()
    }

    /// A one-way compression function built from MiMC.
    ///
    /// This uses the addition variant of Davies-Meyer, unlike MiMChash as described in the MiMC
    /// paper, which uses the sponge framework.
    pub fn mimc_dm_hash(&mut self, x: LinearCombination, y: LinearCombination)
                        -> LinearCombination {
        self.mimc(x, y.clone()) + y
    }
}

fn num_rounds() -> usize {
    let n = FieldElement::max_bits();
    (n as f64 / 3f64.log2()).ceil() as usize
}
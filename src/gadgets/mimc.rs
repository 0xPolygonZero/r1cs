use num::bigint::RandBigInt;

use field_element::FieldElement;
use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;

impl GadgetBuilder {
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
                FieldElement::from(rng.gen_biguint(1000))
            };

            // Add the key and the random round constant to the current value.
            current += key.clone() + round_constant.into();

            // Cube the current value.
            current = self.exp(current, 3);
        }

        // Final key addition, as per the spec.
        current + key.clone()
    }
}

fn num_rounds() -> usize {
    let n = FieldElement::max_bits();
    (n as f64 / 3f64.log2()).ceil() as usize
}
//! This module extends GadgetBuilder with an implementation of MiMC.

use std::borrow::Borrow;

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

use crate::expression::Expression;
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;

impl<F: Field> GadgetBuilder<F> {
    /// The MiMC block cipher.
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
    /// with 0) as the source of randomness.
    pub fn mimc_chacha20(&mut self, key: Expression<F>, input: Expression<F>) -> Expression<F> {
        // Our source of randomness for the round constants will be ChaCha20 with a seed of 0.
        let mut rng = ChaChaRng::seed_from_u64(0);
        let mut round_constants = Vec::new();
        for _r in 0..Self::mimc_recommended_rounds() {
            round_constants.push(Element::random(&mut rng));
        }
        self.mimc(key, input, &round_constants)
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

    #[test]
    fn mimc_f11() {
        #[derive(Debug)]
        struct F11 {}

        impl Field for F11 {
            fn order() -> BigUint {
                BigUint::from(11u8)
            }
        }

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
        struct F7 {}

        impl Field for F7 {
            fn order() -> BigUint {
                BigUint::from(7u8)
            }
        }

        let mut builder = GadgetBuilder::<F7>::new();
        builder.mimc_chacha20(Expression::zero(), Expression::zero());
    }
}
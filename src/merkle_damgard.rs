//! This module extends GadgetBuilder with an implementation of the Merkle-Damgard construction.

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

use crate::{CompressionFunction, HashFunction};
use crate::expression::Expression;
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;

/// A hash function based on the Merkle–Damgård construction.
pub struct MerkleDamgard<F: Field, CF: CompressionFunction<F>> {
    initial_value: Element<F>,
    compress: CF,
}

impl<F: Field, CF: CompressionFunction<F>> MerkleDamgard<F, CF> {
    /// Creates a Merkle–Damgård hash function from the given initial value and one-way compression
    /// function.
    pub fn new(initial_value: Element<F>, compress: CF) -> Self {
        MerkleDamgard { initial_value, compress }
    }

    /// Creates a Merkle–Damgård hash function from the given one-way compression function. Uses
    /// ChaCha20 (seeded with 0) as a source of randomness for the initial value.
    pub fn new_default_initial_value(compress: CF) -> Self {
        let mut rng = ChaChaRng::seed_from_u64(0);
        let initial_value = Element::random(&mut rng);
        Self::new(initial_value, compress)
    }
}

impl<F: Field, C: CompressionFunction<F>> HashFunction<F> for MerkleDamgard<F, C> {
    fn hash(&self, builder: &mut GadgetBuilder<F>, blocks: &[Expression<F>]) -> Expression<F> {
        let mut current = Expression::from(&self.initial_value);
        for block in blocks {
            current = self.compress.compress(builder, &current, block);
        }

        // Length padding
        self.compress.compress(builder, &current, &Expression::from(blocks.len()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{CompressionFunction, HashFunction, MerkleDamgard};
    use crate::expression::Expression;
    use crate::field::{Element, Field};
    use crate::gadget_builder::GadgetBuilder;
    use crate::test_util::F7;

    #[test]
    fn merkle_damgard() {
        // We will use a trivial compression function to keep the test simple.
        struct TestCompress;

        impl<F: Field> CompressionFunction<F> for TestCompress {
            fn compress(
                &self, _builder: &mut GadgetBuilder<F>, x: &Expression<F>, y: &Expression<F>,
            ) -> Expression<F> {
                x * 2 + y * 3
            }
        }

        let mut builder = GadgetBuilder::<F7>::new();
        let x_wire = builder.wire();
        let y_wire = builder.wire();
        let x = Expression::from(x_wire);
        let y = Expression::from(y_wire);
        let blocks = &[x, y];
        let md = MerkleDamgard::new(Element::from(2u8), TestCompress);
        let hash = md.hash(&mut builder, blocks);
        let gadget = builder.build();

        let mut values = values!(x_wire => 3u8.into(), y_wire => 4u8.into());
        assert!(gadget.execute(&mut values));
        // initial value: 2
        // after 3: 2*2 + 3*3 = 6
        // after 4: 6*2 + 4*3 = 3
        // after 2 (length): 3*2 + 2*3 = 5
        assert_eq!(Element::from(5u8), hash.evaluate(&values));
    }
}
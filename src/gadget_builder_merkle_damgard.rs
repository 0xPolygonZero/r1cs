//! This module extends GadgetBuilder with an implementation of the Merkle-Damgard construction.

use core::borrow::Borrow;

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

use crate::expression::Expression;
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;

type CompressionFunction<F> = fn(builder: &mut GadgetBuilder<F>,
                                 x: &Expression<F>,
                                 y: &Expression<F>) -> Expression<F>;

impl<F: Field> GadgetBuilder<F> {
    /// Creates a Merkle–Damgård hash function from the given one-way compression function.
    pub fn merkle_damgard<E>(&mut self, initial_value: E, blocks: &[Expression<F>],
                             compress: CompressionFunction<F>) -> Expression<F>
        where E: Borrow<Element<F>> {
        let mut current = Expression::from(initial_value.borrow());
        let mut len = 0usize;
        for block in blocks {
            current = compress(self, &current, block);
            len += 1;
        }

        // Length padding
        compress(self, &current, &Expression::from(len))
    }

    /// Creates a Merkle–Damgård hash function from the given one-way compression function. Uses
    /// ChaCha20 (seeded with 0) as a source of randomness for the initial value.
    pub fn merkle_damgard_chacha20(&mut self, blocks: &[Expression<F>],
                                   compress: CompressionFunction<F>) -> Expression<F> {
        let mut rng = ChaChaRng::seed_from_u64(0);
        let initial_value = Element::random(&mut rng);
        self.merkle_damgard(initial_value, blocks, compress)
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::{Element, Field};
    use crate::gadget_builder::GadgetBuilder;
    use crate::test_util::F7;

    #[test]
    fn merkle_damgard() {
        // We will use a trivial compression function to keep the test simple.
        fn test_compress<F: Field>(builder: &mut GadgetBuilder<F>,
                                   x: &Expression<F>,
                                   y: &Expression<F>) -> Expression<F> {
            x * 2 + y * 3
        }

        let mut builder = GadgetBuilder::<F7>::new();
        let x_wire = builder.wire();
        let y_wire = builder.wire();
        let x = Expression::from(x_wire);
        let y = Expression::from(y_wire);
        let blocks = &[x, y];
        let md = builder.merkle_damgard(Element::from(2u8), blocks, test_compress);
        let gadget = builder.build();

        let mut values = values!(x_wire => 3u8.into(), y_wire => 4u8.into());
        assert!(gadget.execute(&mut values));
        // initial value: 2
        // after 3: 2*2 + 3*3 = 6
        // after 4: 6*2 + 4*3 = 3
        // after 2 (length): 3*2 + 2*3 = 5
        assert_eq!(Element::from(5u8), md.evaluate(&values));
    }
}
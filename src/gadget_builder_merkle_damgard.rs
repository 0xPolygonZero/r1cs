//! This module extends GadgetBuilder with an implementation of the Merkle-Damgard construction.

use core::borrow::Borrow;

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
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::Bn128;
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn merkle_damgard() {
        let builder = GadgetBuilder::<Bn128>::new();
        // TODO
    }
}
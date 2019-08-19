//! This module extends GadgetBuilder with an implementation of the Davies-Meyer construction.

use core::borrow::Borrow;

use crate::expression::Expression;
use crate::field::Field;
use crate::gadget_builder::GadgetBuilder;

type BlockCipher<F> = fn(builder: &mut GadgetBuilder<F>,
                         key: &Expression<F>,
                         input: &Expression<F>) -> Expression<F>;

impl<F: Field> GadgetBuilder<F> {
    /// Creates a one-way compression function from a block cipher, using the additive variant of
    /// the Davies-Meyer construction.
    pub fn davies_meyer<E1, E2>(&mut self, x: E1, y: E2, cipher: BlockCipher<F>) -> Expression<F>
        where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>> {
        cipher(self, x.borrow(), y.borrow()) + y.borrow()
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::Bn128;
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn davies_meyer() {
        let builder = GadgetBuilder::<Bn128>::new();
        // TODO
    }
}
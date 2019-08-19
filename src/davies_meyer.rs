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
    use crate::field::{Element, Field};
    use crate::gadget_builder::GadgetBuilder;
    use crate::test_util::F7;

    #[test]
    fn davies_meyer() {
        // We will use a trivial cipher to keep the test simple.
        fn test_cipher<F: Field>(builder: &mut GadgetBuilder<F>,
                                 key: &Expression<F>,
                                 input: &Expression<F>) -> Expression<F> {
            let product = builder.product(key, input);
            key * 2 + input * 4 + product * 3
        }

        let mut builder = GadgetBuilder::<F7>::new();
        let x_wire = builder.wire();
        let y_wire = builder.wire();
        let x = Expression::from(x_wire);
        let y = Expression::from(y_wire);
        let dm = builder.davies_meyer(x, y, test_cipher);
        let gadget = builder.build();

        let mut values = values!(x_wire => 2u8.into(), y_wire => 3u8.into());
        assert!(gadget.execute(&mut values));
        // The result should be:
        //   (2x + 4y + 3xy) + y
        // = 2x + 5y + 3xy
        // = 4 + 15 + 18
        // = 37
        // = 2
        assert_eq!(Element::from(2u8), dm.evaluate(&values));
    }
}
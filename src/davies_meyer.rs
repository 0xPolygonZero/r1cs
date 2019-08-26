//! This module extends GadgetBuilder with an implementation of the Davies-Meyer construction.

use std::marker::PhantomData;

use crate::expression::Expression;
use crate::field::Field;
use crate::gadget_builder::GadgetBuilder;
use crate::gadget_traits::{BlockCipher, CompressionFunction};

/// The additive variant of Davies-Meyer, which creates a one-way compression function from a block
/// cipher.
pub struct DaviesMeyer<F: Field, BC: BlockCipher<F>> {
    cipher: BC,
    phantom: PhantomData<F>,
}

impl<F: Field, BC: BlockCipher<F>> DaviesMeyer<F, BC> {
    /// Create a new Davies-Meyer compression function from the given block cipher.
    pub fn new(cipher: BC) -> Self {
        DaviesMeyer { cipher, phantom: PhantomData }
    }
}

impl<F: Field, BC: BlockCipher<F>> CompressionFunction<F> for DaviesMeyer<F, BC> {
    fn compress(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>, y: &Expression<F>)
                -> Expression<F> {
        self.cipher.encrypt(builder, x, y) + y
    }
}

#[cfg(test)]
mod tests {
    use crate::davies_meyer::DaviesMeyer;
    use crate::expression::Expression;
    use crate::field::{Element, Field};
    use crate::gadget_builder::GadgetBuilder;
    use crate::gadget_traits::{BlockCipher, CompressionFunction};
    use crate::test_util::F7;

    #[test]
    fn davies_meyer() {
        // We will use a trivial cipher to keep the test simple.
        struct TestCipher;

        impl<F: Field> BlockCipher<F> for TestCipher {
            fn encrypt(&self, builder: &mut GadgetBuilder<F>, key: &Expression<F>,
                       input: &Expression<F>) -> Expression<F> {
                let product = builder.product(key, input);
                key * 2 + input * 4 + product * 3
            }

            fn decrypt(&self, _builder: &mut GadgetBuilder<F>, _key: &Expression<F>,
                       _output: &Expression<F>) -> Expression<F> {
                panic!("Should never be called")
            }
        }

        let mut builder = GadgetBuilder::<F7>::new();
        let x_wire = builder.wire();
        let y_wire = builder.wire();
        let x = Expression::from(x_wire);
        let y = Expression::from(y_wire);
        let dm = DaviesMeyer::new(TestCipher);
        let dm_output = dm.compress(&mut builder, &x, &y);
        let gadget = builder.build();

        let mut values = values!(x_wire => 2u8.into(), y_wire => 3u8.into());
        assert!(gadget.execute(&mut values));
        // The result should be:
        //   (2x + 4y + 3xy) + y
        // = 2x + 5y + 3xy
        // = 4 + 15 + 18
        // = 37
        // = 2
        assert_eq!(Element::from(2u8), dm_output.evaluate(&values));
    }
}
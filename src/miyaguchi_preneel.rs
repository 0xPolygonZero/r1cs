use std::marker::PhantomData;

use crate::expression::Expression;
use crate::field::Field;
use crate::gadget_builder::GadgetBuilder;
use crate::gadget_traits::{BlockCipher, CompressionFunction};

/// The additive variant of Miyaguchi-Preneel, which creates a one-way compression function from a
/// block cipher.
pub struct MiyaguchiPreneel<F: Field, BC: BlockCipher<F>> {
    cipher: BC,
    phantom: PhantomData<*const F>,
}

impl<F: Field, BC: BlockCipher<F>> MiyaguchiPreneel<F, BC> {
    /// Create a new Miyaguchi-Preneel compression function from the given block cipher.
    pub fn new(cipher: BC) -> Self {
        MiyaguchiPreneel { cipher, phantom: PhantomData }
    }
}

impl<F: Field, BC: BlockCipher<F>> CompressionFunction<F> for MiyaguchiPreneel<F, BC> {
    fn compress(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>, y: &Expression<F>)
                -> Expression<F> {
        self.cipher.encrypt(builder, x, y) + x + y
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::{Element, Field};
    use crate::gadget_builder::GadgetBuilder;
    use crate::gadget_traits::{BlockCipher, CompressionFunction};
    use crate::miyaguchi_preneel::MiyaguchiPreneel;
    use crate::test_util::F7;

    #[test]
    fn miyaguchi_preneel() {
        // We will use a trivial cipher to keep the test simple.
        // The cipher is: (k, i) -> 2k + 4i + 3ki
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
        let mp = MiyaguchiPreneel::new(TestCipher);
        let mp_output = mp.compress(&mut builder, &x, &y);
        let gadget = builder.build();

        let mut values = values!(x_wire => 2u8.into(), y_wire => 3u8.into());
        assert!(gadget.execute(&mut values));
        // The result should be: (2x + 4y + 3xy) + x + y = 4 + 12 + 18 + 2 + 3 = 39 = 4.
        assert_eq!(Element::from(4u8), mp_output.evaluate(&values));
    }
}
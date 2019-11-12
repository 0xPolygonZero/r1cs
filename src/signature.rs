use std::marker::PhantomData;

use crate::{CompressionFunction, CyclicGenerator, Expression, Field, GadgetBuilder, GroupExpression, Group};

pub trait SignatureScheme<F: Field, G: Group<F>, C: CyclicGenerator<F, G>, CF> {
    fn verify(
        builder: &mut GadgetBuilder<F>,
        signature: &SignatureExpression<F>,
        message: &Expression<F>,
        public_key: &G::GroupExpression,
        compress: &CF,
    ) where CF: CompressionFunction<F>;
}

pub struct Schnorr<F: Field, G: Group<F>, C: CyclicGenerator<F, G>, CF>  {
    phantom_f: PhantomData<*const F>,
    phantom_g: PhantomData<*const G>,
    phantom_c: PhantomData<*const C>,
    phantom_cf: PhantomData<*const CF>,
}

/// Struct to represent a Schnorr Signature.
///
/// Assumes that the message has already been hashed to a field element
/// Signature is a tuple consisting of scalars (s, e), where r_v = sg + ey
/// Public key is a group element, y = xg for private key x
pub struct SignatureExpression<F: Field> {
    pub s: Expression<F>,
    pub e: Expression<F>,
}

impl<F: Field, G: Group<F>, C: CyclicGenerator<F, G>, CF> SignatureScheme<F, G, C, CF> for Schnorr<F, G, C, CF> {
    /// Generates constraints to verify that a Schnorr signature for a message is valid,
    /// given a public key and a secure compression function.
    ///
    /// Requires a preimage-resistant hash function for full security.
    ///
    /// A naive implementation that has not been optimized or audited.
    // TODO: optimize scalar multiplication for a fixed generator
    fn verify(
        builder: &mut GadgetBuilder<F>,
        signature: &SignatureExpression<F>,
        message: &Expression<F>,
        public_key: &G::GroupExpression,
        compress: &CF,
    ) where CF: CompressionFunction<F> {
        let generator = C::generator_expression();
        let gs = G::mul_scalar_expression(
            builder,
            &generator,
            &signature.s);
        let ye = G::mul_scalar_expression(
            builder,
            public_key,
            &signature.e);
        let gs_ye = G::add_expressions(builder, &gs, &ye);

        // TODO: verify that compressing the Edwards Curve point to the Y-coordinate is valid
        let hash_check = compress.compress(builder, &gs_ye.compressed(), &message);
        builder.assert_equal(&hash_check, &signature.e);
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{CyclicGenerator, EdwardsExpression, EdwardsPoint, Expression, GadgetBuilder, Group, WireValues, JubJub, EdwardsGroup};
    use crate::CompressionFunction;
    use crate::EdwardsCurve;
    use crate::field::{Bls12_381, Element, Field};
    use crate::JubJubGenerator;
    use crate::signature::{Schnorr, SignatureExpression, SignatureScheme};

    #[test]
    fn verify() {
        // Generate signature
        let generator = JubJubGenerator::generator_element();

        let private_key = Element::from_str("4372820819045374670962167435360035096875258").unwrap();

        let mut builder = GadgetBuilder::<Bls12_381>::new();

        let public_key
            = EdwardsGroup::<Bls12_381, JubJub>::mul_scalar_element(&generator, &private_key);

        let nonce = Element::from_str("5434290453746709621674353600312312").unwrap();

        let r
            = EdwardsGroup::<Bls12_381, JubJub>::mul_scalar_element(&generator, &nonce);

        let compress = TestCompress {};

        let message = Element::from_str("12345").unwrap();

        let e = compress.compress_evaluate(&r.compressed_element(), &message);

        let s = &nonce - &private_key * &e;

        let signature = SignatureExpression { s: Expression::from(s), e: Expression::from(e) };

        let mut builder = GadgetBuilder::<Bls12_381>::new();

        Schnorr::<Bls12_381, EdwardsGroup<Bls12_381, JubJub>, JubJubGenerator, TestCompress>::verify(
            &mut builder,
            &signature,
            &Expression::from(message),
            &EdwardsExpression::from(&public_key),
            &compress,
        );

        let gadget = builder.build();
        let mut values = WireValues::new();
        gadget.execute(&mut values);

        //TODO: include test vectors
    }

    // A dummy compression function which returns 2x + y.
    struct TestCompress;

    impl<F: Field> CompressionFunction<F> for TestCompress {
        fn compress(&self, _builder: &mut GadgetBuilder<F>, x: &Expression<F>, y: &Expression<F>)
                    -> Expression<F> {
            x * 2 + y
        }
    }
}
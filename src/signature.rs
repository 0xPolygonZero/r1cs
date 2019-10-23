use crate::{CompressionFunction, Compressible, CyclicGroup, Expression, Field, GadgetBuilder};

pub trait SignatureScheme<F: Field, C: CyclicGroup<F>, CF> {
    fn verify(
        builder: &mut GadgetBuilder<F>,
        signature: &SignatureExpression<F>,
        message: &Expression<F>,
        public_key: &C::GroupExpression,
        compress: &CF,
    ) where CF: CompressionFunction<F>;
}

pub struct Schnorr {
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

impl<F: Field, C: CyclicGroup<F>, CF> SignatureScheme<F, C, CF> for Schnorr {
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
        public_key: &C::GroupExpression,
        compress: &CF,
    ) where CF: CompressionFunction<F> {
        let generator = C::generator_expression();
        let gs = C::scalar_mult_expression(
            builder,
            &generator,
            &signature.s);
        let ye = C::scalar_mult_expression(
            builder,
            public_key,
            &signature.e);
        let gs_ye = C::add_expressions(builder, &gs, &ye);

        // TODO: verify that compressing the Edwards Curve point to the Y-coordinate is valid
        let hash_check = compress.compress(builder, &gs_ye.compressed_expression(), &message);
        builder.assert_equal(&hash_check, &signature.e);
    }
}

/*
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{EdwardsPointExpression, Expression, GadgetBuilder, WireValues, EdwardsPoint};
    use crate::CompressionFunction;
    use crate::curves::EdwardsCurve;
    use crate::embedded_curve::JubJub;
    use crate::field::{Bls12_381, Element, Field};
    use crate::signature::{SchnorrSignatureExpression, SignatureExpression};

    #[test]
    fn verify() {
        // Generate signature
        let generator = EdwardsPoint::<Bls12_381, JubJub>::from_elements(
            JubJub::subgroup_generator().0, JubJub::subgroup_generator().1,
        );

        let private_key = Element::from_str("4372820819045374670962167435360035096875258").unwrap();

        let mut builder = GadgetBuilder::<Bls12_381>::new();

        let public_key = generator.scalar_mult_evaluate(&private_key);

        let nonce = Element::from_str("5434290453746709621674353600312312").unwrap();

        let r = generator.scalar_mult_evaluate(&nonce);

        let compress = TestCompress {};

        let message = Element::from_str("12345").unwrap();

        let e = compress.compress_evaluate(&r.compressed(), &message);

        let s = &nonce - &private_key * &e;

        let signature = SchnorrSignatureExpression { s: Expression::from(s), e: Expression::from(e) };

        let mut builder = GadgetBuilder::<Bls12_381>::new();

        signature.verify(
            &mut builder,
            &Expression::from(message),
            &EdwardsPointExpression::from_edwards_point(public_key),
            &compress,
        );

        let gadget = builder.build();
        let mut values = WireValues::new();
        gadget.execute(&mut values);
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
*/
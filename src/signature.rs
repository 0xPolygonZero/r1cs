
use crate::{CompressionFunction, EdwardsCurve, EdwardsPointExpression, Expression, Field, GadgetBuilder};

pub trait SignatureExpression<F: Field, C: EdwardsCurve<F>, CF> {
    fn verify(
        &self,
        builder: &mut GadgetBuilder<F>,
        message: &Expression<F>,
        public_key: &EdwardsPointExpression<F, C>,
        compress: &CF,
    ) where CF: CompressionFunction<F>;
}

/// Struct to represent a Schnorr Signature.
///
/// Assumes that the message has already been hashed to a field element
/// Signature is a tuple consisting of scalars (s, e), where r_v = sg + ey
/// Public key is a group element, y = xg for private key x
pub struct SchnorrSignatureExpression<F: Field> {
    pub s: Expression<F>,
    pub e: Expression<F>,
}

impl<F: Field, C: EdwardsCurve<F>, CF> SignatureExpression<F, C, CF> for SchnorrSignatureExpression<F> {
    /// Generates constraints to verify that a Schnorr signature for a message is valid,
    /// given a public key and a secure compression function.
    ///
    /// Requires a preimage-resistant hash function for full security.
    ///
    /// A naive implementation that has not been optimized or audited.
    // TODO: optimize scalar multiplication for a fixed generator
    fn verify(
        &self,
        builder: &mut GadgetBuilder<F>,
        message: &Expression<F>,
        public_key: &EdwardsPointExpression<F, C>,
        compress: &CF,
    ) where CF: CompressionFunction<F> {
        let generator = EdwardsPointExpression::from_elements(
            C::subgroup_generator().0, C::subgroup_generator().1,
        );
        let gs = EdwardsPointExpression::scalar_mult(builder, &generator, &self.s);
        let ye = EdwardsPointExpression::scalar_mult(builder, public_key, &self.e);
        let gs_ye = EdwardsPointExpression::add(builder, &gs, &ye);

        // TODO: verify that compressing the Edwards Curve point to the Y-coordinate is valid
        let hash_check = compress.compress(builder, gs_ye.compressed(), &message);
        builder.assert_equal(&hash_check, &self.e);
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{EdwardsPointExpression, Expression, GadgetBuilder, WireValues};
    use crate::CompressionFunction;
    use crate::curve::EdwardsCurve;
    use crate::embedded_curve::JubJub;
    use crate::field::{Bls12_381, Element, Field};
    use crate::signature::{SchnorrSignatureExpression, SignatureExpression};

    #[test]
    fn verify() {
        // Generate signature
        let generator = EdwardsPointExpression::<Bls12_381, JubJub>::from_elements(
            JubJub::subgroup_generator().0, JubJub::subgroup_generator().1,
        );

        let private_key = Expression::<Bls12_381>::from(Element::from_str(
            "4372820819045374670962167435360035096875258"
        ).unwrap());

        let mut builder = GadgetBuilder::<Bls12_381>::new();

        let public_key = EdwardsPointExpression::<Bls12_381, JubJub>::scalar_mult(
            &mut builder, &generator, &private_key,
        );

        let nonce = Expression::<Bls12_381>::from(Element::from_str(
            "5434290453746709621674353600312312"
        ).unwrap());

        let r = EdwardsPointExpression::<Bls12_381, JubJub>::scalar_mult(
            &mut builder, &generator, &nonce,
        );

        let compress = TestCompress {};

        let message = Expression::<Bls12_381>::from(Element::from_str(
            "12345"
        ).unwrap());

        let e = compress.compress(&mut builder, &r.compressed(), &message);

        let gadget = builder.build();
        let mut values = WireValues::new();
        gadget.execute(&mut values);

        let s = Expression::<Bls12_381>::from(
            nonce.evaluate(&values) - private_key.evaluate(&values) * e.evaluate(&values)
        );

        let signature = SchnorrSignatureExpression { s, e };

        let mut sig_builder = GadgetBuilder::<Bls12_381>::new();

        signature.verify(&mut sig_builder, &message, &public_key, &compress);

        let sig_gadget = sig_builder.build();
        let mut sig_values = WireValues::new();
        sig_gadget.execute(&mut sig_values);
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
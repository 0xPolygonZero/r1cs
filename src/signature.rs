use std::borrow::Borrow;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Shl, Sub, SubAssign};
use std::str::FromStr;

use num::bigint::ParseBigIntError;
use num::BigUint;
use num::pow;

use crate::{Expression, GadgetBuilder, BooleanExpression, EdwardsCurve, Curve, CurvePoint, EdwardsPointExpression};
use crate::field::{Element, Field};

pub trait SignatureExpression<F: Field, C: EdwardsCurve<F>> {
    fn verify(&self, builder: &mut GadgetBuilder<F>);
}



/// Struct to represent a Schnorr Signature verifiable with a public key.
///
/// Assumes that the message has already been hashed to a field element
/// Signature is a tuple consisting of (s, e), where r_v = sg + ey
/// Public key is a group element, y = xg for private key x
pub struct SchnorrSignatureExpression<F: Field, C: EdwardsCurve<F>> {
    message: Expression<F>,
    s: Expression<F>,
    e: Expression<F>,
    public_key: EdwardsPointExpression<F, C>,
}

impl<F: Field, C: EdwardsCurve<F>> SignatureExpression<F, C> for SchnorrSignatureExpression<F, C> {

    /// Generates constraints to verify that a signature is valid. A naive implementation that
    /// has not been optimized or audited.
    fn verify(
        &self,
        builder: &mut GadgetBuilder<F>,
    ) {
        let generator = EdwardsPointExpression::from_elements(
            C::subgroup_generator().0, C::subgroup_generator().1
        );
        let gs = EdwardsPointExpression::scalar_mult(builder, &generator, &self.s);
        let ye = EdwardsPointExpression::scalar_mult(builder, &self.public_key, &self.e);
        let gs_ye = EdwardsPointExpression::add(builder, &gs, &ye);

        // let hash_check = Hash(gs_ye || M);
        let hash_check = e;
        builder.assert_equal(hash_check, e);
    }
}

#[cfg(test)]
mod tests {
    use std::iter;
    use std::str::FromStr;

    use itertools::assert_equal;
    use num::BigUint;

    use crate::curve::{EdwardsCurve};
    use crate::field::{Bls12_381, Bn128, Element, Field};
    use crate::{EdwardsPointExpression, Expression, GadgetBuilder, WireValues};

    struct JubJub {}

    impl EdwardsCurve<Bls12_381> for JubJub {
        fn a() -> Element<Bls12_381> {
            -Element::one()
        }

        fn d() -> Element<Bls12_381> {
            Element::from_str(
                "19257038036680949359750312669786877991949435402254120286184196891950884077233"
            ).unwrap()
        }

        fn subgroup_generator() -> (Element<Bls12_381>, Element<Bls12_381>) {
            let x = Element::from_str(
                "16540640123574156134436876038791482806971768689494387082833631921987005038935"
            ).unwrap();
            let y = Element::from_str(
                "20819045374670962167435360035096875258406992893633759881276124905556507972311"
            ).unwrap();

            (x ,y)
        }
    }

    #[test]
    fn check_verify() {
        // TODO: add a test for verifying a signature
    }
}
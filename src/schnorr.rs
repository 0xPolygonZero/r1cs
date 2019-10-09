use std::borrow::Borrow;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Shl, Sub, SubAssign};
use std::str::FromStr;

use num::bigint::ParseBigIntError;
use num::BigUint;
use num::pow;

use crate::{Expression, GadgetBuilder, BooleanExpression};
use crate::field::{Element, Field};



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
    }

    #[test]
    fn check_verify() {
        // TODO: add a test for verifying a signature
    }
}
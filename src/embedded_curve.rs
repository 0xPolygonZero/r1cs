//! Families of embedded curves, defined over
//! the same base field as the constraint system.

use std::str::FromStr;

use crate::{AffineCurvePoint, Bls12_381, Element, Field, TwistedEdwardsCurve};

pub fn jubjub<F: Field>() -> TwistedEdwardsCurve<F> {
    TwistedEdwardsCurve {
        a: -Element::one(),
        d: Element::from_str(
            "19257038036680949359750312669786877991949435402254120286184196891950884077233"
        ).unwrap(),
    }
}

pub fn jubjub_prime_subgroup<F: Field>() -> Subgroup<F, TwistedEdwardsCurve<F>> {
    let x = Element::from_str(
        "11076627216317271660298050606127911965867021807910416450833192264015104452986"
    ).unwrap();
    let y = Element::from_str(
        "44412834903739585386157632289020980010620626017712148233229312325549216099227"
    ).unwrap();

    Subgroup {
        group: jubjub(),
        generator: AffineCurvePoint { x, y },
    }
}

#[cfg(test)]
mod tests {
    use std::iter;
    use std::str::FromStr;

    use itertools::assert_equal;

    use crate::field::{Bls12_381, Bn128, Element};

    #[test]
    fn subgroup_check() {
        //TODO: verify that generator is valid and generates a subgroup of prime order with appropriate cofactor
    }
}

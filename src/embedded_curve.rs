use std::str::FromStr;

use crate::{Bls12_381, EdwardsCurve, Element};

// Families of embedded curves, defined over
// the same base field as the constraint system.

pub struct JubJub {}

impl EdwardsCurve<Bls12_381> for JubJub {
    fn a() -> Element<Bls12_381> {
        -Element::one()
    }

    fn d() -> Element<Bls12_381> {
        Element::from_str(
            "19257038036680949359750312669786877991949435402254120286184196891950884077233"
        ).unwrap()
    }

    // TODO: determine whether this is the correct generator for the JubJub prime order subgroup
    fn subgroup_generator() -> (Element<Bls12_381>, Element<Bls12_381>) {
        let x = Element::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y = Element::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use std::iter;
    use std::str::FromStr;

    use itertools::assert_equal;

    use crate::field::{Bn128, Bls12_381, Element};

    #[test]
    fn subgroup_check() {
        //TODO: verify that generator is valid and generates a subgroup of prime order with appropriate cofactor
    }
}

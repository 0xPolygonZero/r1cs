use crate::{Curve, EdwardsCurve, Element, Bls12_381, EdwardsPoint, Group, CyclicGroup, EdwardsExpression};

pub struct JubJub {}

impl Curve<Bls12_381> for JubJub {}

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

impl CyclicGroup<Bls12_381> for JubJub {
    fn generator_element() -> EdwardsPoint<Bls12_381, JubJub> {
        let x = Element::from(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y = Element::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        (x, y)
    }

    fn generator_expression() -> EdwardsExpression<Bls12_381, JubJub> {

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

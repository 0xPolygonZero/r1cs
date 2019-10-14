use std::str::FromStr;

use crate::{AffineTwistedEdwardsCurve, AffineTwistedEdwardsPoint, Bls12_381, CyclicSubgroup, TwistedEdwardsCurveParams, Element, GroupGenerator, PrimeOrderGroup};

type JubJub = AffineTwistedEdwardsCurve<Bls12_381, JubJubParams>;

pub struct JubJubParams;

impl TwistedEdwardsCurveParams<Bls12_381> for JubJubParams {
    fn a() -> Element<Bls12_381> {
        -Element::one()
    }

    fn d() -> Element<Bls12_381> {
        Element::from_str(
            "19257038036680949359750312669786877991949435402254120286184196891950884077233"
        ).unwrap()
    }
}

pub struct JubJubPrimeSubgroupGenerator;

impl GroupGenerator<AffineTwistedEdwardsPoint<Bls12_381, JubJubParams>> for JubJubPrimeSubgroupGenerator {
    fn generator() -> AffineTwistedEdwardsPoint<Bls12_381, JubJubParams> {
        let x = Element::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y = Element::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();
        AffineTwistedEdwardsPoint::new(x, y)
    }
}

pub type JubJubPrimeSubgroup = CyclicSubgroup<Bls12_381, JubJub, JubJubPrimeSubgroupGenerator>;

impl PrimeOrderGroup<Bls12_381> for JubJubPrimeSubgroup {}

#[cfg(test)]
mod tests {
    use crate::{AffineTwistedEdwardsPoint, Element, Group, JubJubPrimeSubgroup};

    #[test]
    fn test_double() {
        let x = Element::zero();
        let y = Element::zero();
        let p = AffineTwistedEdwardsPoint::new(x, y);
        let two_p = JubJubPrimeSubgroup::double_element(&p);
        // TODO: Finish
    }
}
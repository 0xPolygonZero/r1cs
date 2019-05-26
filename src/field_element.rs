use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg};
use std::str::FromStr;

use num::bigint::ParseBigIntError;
use num::BigUint;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FieldElement {
    value: BigUint,
}

impl FieldElement {
    /// The prime field size.
    pub fn size() -> BigUint {
        BigUint::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495617").unwrap()
    }

    pub fn max_value() -> BigUint {
        FieldElement::size() - BigUint::from(1u64)
    }

    /// The number of bits needed to encode each field element.
    pub fn bits() -> usize {
        FieldElement::max_value().bits()
    }

    pub fn zero() -> Self {
        FieldElement::from(0)
    }

    pub fn one() -> Self {
        FieldElement::from(1)
    }

    pub fn multiplicative_inverse(&self) -> FieldElement {
        assert_ne!(*self, FieldElement::zero(), "Zero does not have a multiplicative inverse");
        // From Euler's theorem.
        FieldElement::from(self.value.modpow(
            &(FieldElement::size() - BigUint::from(2u64)),
            &FieldElement::size()))
    }

    /// Return the i'th least significant bit. So, for example, x.bit(0) returns the least
    /// significant bit of x.
    pub fn bit(&self, i: usize) -> bool {
        (self.value.clone() >> i) & BigUint::from(1u64) == BigUint::from(1u64)
    }
}

impl From<BigUint> for FieldElement {
    fn from(value: BigUint) -> FieldElement {
        assert!(value >= BigUint::from(0u64));
        assert!(value < FieldElement::size());
        FieldElement { value }
    }
}

impl From<u128> for FieldElement {
    fn from(value: u128) -> FieldElement {
        FieldElement::from(BigUint::from(value))
    }
}

impl FromStr for FieldElement {
    type Err = ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigUint::from_str(s).map(|n| FieldElement::from(n))
    }
}

impl Neg for FieldElement {
    type Output = FieldElement;

    fn neg(self) -> FieldElement {
        if self == FieldElement::zero() {
            self
        } else {
            FieldElement::from(FieldElement::size() - self.value)
        }
    }
}

impl Add<FieldElement> for FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: FieldElement) -> FieldElement {
        FieldElement::from((self.value + rhs.value) % FieldElement::size())
    }
}

impl AddAssign for FieldElement {
    fn add_assign(&mut self, rhs: FieldElement) {
        *self = self.clone() + rhs;
    }
}

impl Mul<FieldElement> for FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: FieldElement) -> FieldElement {
        FieldElement::from((self.value * rhs.value) % FieldElement::size())
    }
}

impl Mul<u128> for FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: u128) -> FieldElement {
        self * FieldElement::from(rhs)
    }
}

impl MulAssign for FieldElement {
    fn mul_assign(&mut self, rhs: FieldElement) {
        *self = self.clone() * rhs;
    }
}

impl MulAssign<u128> for FieldElement {
    fn mul_assign(&mut self, rhs: u128) {
        *self = self.clone() * rhs;
    }
}

impl Div<FieldElement> for FieldElement {
    type Output = FieldElement;

    fn div(self, rhs: FieldElement) -> FieldElement {
        self * rhs.multiplicative_inverse()
    }
}

#[cfg(test)]
mod tests {
    use std::iter;
    use std::str::FromStr;

    use itertools::assert_equal;

    use field_element::FieldElement;

    #[test]
    fn addition() {
        assert_eq!(
            FieldElement::from(2),
            FieldElement::one() + FieldElement::one());

        assert_eq!(
            FieldElement::from(33),
            FieldElement::from(13) + FieldElement::from(20));
    }

    #[test]
    fn addition_overflow() {
        assert_eq!(
            FieldElement::from_str("3").unwrap(),
            FieldElement::from_str(
                "21888242871839275222246405745257275088548364400416034343698204186575808495615"
            ).unwrap() + FieldElement::from_str("5").unwrap());
    }

    #[test]
    fn additive_inverse() {
        assert_eq!(
            FieldElement::from_str(
                "21888242871839275222246405745257275088548364400416034343698204186575808495616"
            ).unwrap(),
            -FieldElement::one());

        assert_eq!(
            FieldElement::zero(),
            FieldElement::from(123) + -FieldElement::from(123));
    }

    #[test]
    fn multiplication_overflow() {
        assert_eq!(
            FieldElement::from_str(
                "13869117166973684714533159833916213390696312133829829072325816326144232854527"
            ).unwrap(),
            FieldElement::from_str("1234567890123456789012345678901234567890").unwrap()
                * FieldElement::from_str("1234567890123456789012345678901234567890").unwrap());
    }

    #[test]
    fn bits_0() {
        let x = FieldElement::from(0);
        let n: usize = 300;
        assert_equal(
            iter::repeat(false).take(n),
            (0..n).map(|i| x.bit(i)));
    }

    #[test]
    fn bits_19() {
        let x = FieldElement::from(19);
        assert_eq!(true, x.bit(0));
        assert_eq!(true, x.bit(1));
        assert_eq!(false, x.bit(2));
        assert_eq!(false, x.bit(3));
        assert_eq!(true, x.bit(4));
        assert_eq!(false, x.bit(5));
        assert_eq!(false, x.bit(6));
    }
}

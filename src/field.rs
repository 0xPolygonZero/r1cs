use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Shl, Sub, SubAssign};
use std::str::FromStr;

use num::bigint::ParseBigIntError;
use num::bigint::RandBigInt;
use num::BigUint;
use num_traits::One;
use num_traits::Zero;
use rand::Rng;

/// A prime order field.
pub trait Field: 'static {
    /// The (prime) order of this field.
    fn order() -> BigUint;
}

/// The BN128 curve.
#[derive(Debug)]
pub struct Bn128 {}

impl Field for Bn128 {
    fn order() -> BigUint {
        BigUint::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495617"
        ).unwrap()
    }
}

/// The BLS12-381 curve.
#[derive(Debug)]
pub struct Bls12_381 {}

impl Field for Bls12_381 {
    fn order() -> BigUint {
        BigUint::from_str(
            "52435875175126190479447740508185965837690552500527637822603658699938581184513"
        ).unwrap()
    }
}

/// An element of a prime field.
#[derive(Debug)]
pub struct Element<F: Field> {
    n: BigUint,
    /// F needs to be present in a struct field, otherwise the compiler will complain that it is
    /// unused. In reality it is used, but only at compile time. For example, some functions take an
    /// `Element<F>` and call `F::order()`.
    phantom: PhantomData<*const F>,
}

impl<F: Field> Element<F> {
    pub fn zero() -> Self {
        Self::from(BigUint::zero())
    }

    pub fn one() -> Self {
        Self::from(BigUint::one())
    }

    pub fn largest_element() -> Self {
        Self::from(F::order() - BigUint::one())
    }

    pub fn to_biguint(&self) -> &BigUint {
        &self.n
    }

    pub fn is_zero(&self) -> bool {
        self.to_biguint().is_zero()
    }

    pub fn is_nonzero(&self) -> bool {
        !self.to_biguint().is_zero()
    }

    pub fn is_one(&self) -> bool {
        self.to_biguint().is_one()
    }

    pub fn multiplicative_inverse(&self) -> Self {
        assert!(!self.is_zero(), "Zero does not have a multiplicative inverse");
        // From Fermat's little theorem.
        // TODO: Use a faster method, like the one described in "Fast Modular Reciprocals".
        // Or just wait for https://github.com/rust-num/num-bigint/issues/60
        self.exponentiation(&-Self::from(2u8))
    }

    /// Like `multiplicative_inverse`, except that zero is mapped to itself rather than causing a
    /// panic.
    pub fn multiplicative_inverse_or_zero(&self) -> Self {
        if self.is_zero() {
            Self::zero()
        } else {
            self.multiplicative_inverse()
        }
    }

    pub fn exponentiation(&self, power: &Self) -> Self {
        Self::from(self.to_biguint().modpow(power.to_biguint(), &F::order()))
    }

    pub fn integer_division(&self, rhs: &Self) -> Self {
        Self::from(self.to_biguint() / rhs.to_biguint())
    }

    pub fn integer_modulus(&self, rhs: &Self) -> Self {
        Self::from(self.to_biguint() % rhs.to_biguint())
    }

    pub fn gcd(&self, rhs: &Self) -> Self {
        // This is just the Euclidean algorithm.
        if rhs.is_zero() {
            self.clone()
        } else {
            rhs.gcd(&self.integer_modulus(rhs))
        }
    }

    /// The number of bits needed to encode every element of `F`.
    pub fn max_bits() -> usize {
        Self::largest_element().bits()
    }

    /// The number of bits needed to encode this particular field element.
    pub fn bits(&self) -> usize {
        self.to_biguint().bits()
    }

    /// Return the i'th least significant bit. So, for example, x.bit(0) returns the least
    /// significant bit of x. Return false for outside of range.
    pub fn bit(&self, i: usize) -> bool {
        ((self.to_biguint() >> i) & BigUint::one()).is_one()
    }

    /// Return a random field element, uniformly distributed in [0, size()).
    /// This is the fastest implementation since max_bits() is always GSB bounded.
    pub fn random(rng: &mut impl Rng) -> Self {
        let bits = Self::max_bits();
        loop {
            let r = rng.gen_biguint(bits);
            if r < F::order() {
                return Self::from(r);
            }
        }
    }
}

impl<F: Field> From<BigUint> for Element<F> {
    fn from(n: BigUint) -> Element<F> {
        assert!(n < F::order(), "Out of range");
        Element { n, phantom: PhantomData }
    }
}

impl<F: Field> From<usize> for Element<F> {
    fn from(n: usize) -> Element<F> {
        Element::from(BigUint::from(n))
    }
}

impl<F: Field> From<u128> for Element<F> {
    fn from(n: u128) -> Element<F> {
        Element::from(BigUint::from(n))
    }
}

impl<F: Field> From<u64> for Element<F> {
    fn from(n: u64) -> Element<F> {
        Element::from(BigUint::from(n))
    }
}

impl<F: Field> From<u32> for Element<F> {
    fn from(n: u32) -> Element<F> {
        Element::from(BigUint::from(n))
    }
}

impl<F: Field> From<u16> for Element<F> {
    fn from(n: u16) -> Element<F> {
        Element::from(BigUint::from(n))
    }
}

impl<F: Field> From<u8> for Element<F> {
    fn from(n: u8) -> Element<F> {
        Element::from(BigUint::from(n))
    }
}

impl<F: Field> From<bool> for Element<F> {
    fn from(b: bool) -> Element<F> {
        Element::from(b as u128)
    }
}

impl<F: Field> FromStr for Element<F> {
    type Err = ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigUint::from_str(s).map(Element::from)
    }
}

impl<F: Field> PartialEq for Element<F> {
    fn eq(&self, other: &Self) -> bool {
        self.to_biguint() == other.to_biguint()
    }
}

impl<F: Field> Eq for Element<F> {}

impl<F: Field> Clone for Element<F> {
    fn clone(&self) -> Self {
        Element::from(self.to_biguint().clone())
    }
}

impl<F: Field> Hash for Element<F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.n.hash(state)
    }
}

impl<F: Field> Ord for Element<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.n.cmp(&other.n)
    }
}

impl<F: Field> PartialOrd for Element<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: Field> Neg for Element<F> {
    type Output = Element<F>;

    fn neg(self) -> Element<F> {
        -&self
    }
}

impl<F: Field> Neg for &Element<F> {
    type Output = Element<F>;

    fn neg(self) -> Element<F> {
        if self.is_zero() {
            Element::zero()
        } else {
            Element::from(F::order() - self.to_biguint())
        }
    }
}

impl<F: Field> Add<Element<F>> for Element<F> {
    type Output = Element<F>;

    fn add(self, rhs: Element<F>) -> Element<F> {
        &self + &rhs
    }
}

impl<F: Field> Add<&Element<F>> for Element<F> {
    type Output = Element<F>;

    fn add(self, rhs: &Element<F>) -> Element<F> {
        &self + rhs
    }
}

impl<F: Field> Add<Element<F>> for &Element<F> {
    type Output = Element<F>;

    fn add(self, rhs: Element<F>) -> Element<F> {
        self + &rhs
    }
}

impl<F: Field> Add<&Element<F>> for &Element<F> {
    type Output = Element<F>;

    fn add(self, rhs: &Element<F>) -> Element<F> {
        Element::from((self.to_biguint() + rhs.to_biguint()) % F::order())
    }
}

impl<F: Field> AddAssign for Element<F> {
    fn add_assign(&mut self, rhs: Element<F>) {
        *self += &rhs;
    }
}

impl<F: Field> AddAssign<&Element<F>> for Element<F> {
    fn add_assign(&mut self, rhs: &Element<F>) {
        *self = &*self + rhs;
    }
}

impl<F: Field> Sub<Element<F>> for Element<F> {
    type Output = Element<F>;

    fn sub(self, rhs: Element<F>) -> Element<F> {
        &self - &rhs
    }
}

impl<F: Field> Sub<&Element<F>> for Element<F> {
    type Output = Element<F>;

    fn sub(self, rhs: &Element<F>) -> Element<F> {
        &self - rhs
    }
}

impl<F: Field> Sub<Element<F>> for &Element<F> {
    type Output = Element<F>;

    fn sub(self, rhs: Element<F>) -> Element<F> {
        self - &rhs
    }
}

impl<F: Field> Sub<&Element<F>> for &Element<F> {
    type Output = Element<F>;

    fn sub(self, rhs: &Element<F>) -> Element<F> {
        self + -rhs
    }
}

impl<F: Field> SubAssign for Element<F> {
    fn sub_assign(&mut self, rhs: Element<F>) {
        *self -= &rhs;
    }
}

impl<F: Field> SubAssign<&Element<F>> for Element<F> {
    fn sub_assign(&mut self, rhs: &Element<F>) {
        *self = &*self - rhs;
    }
}

impl<F: Field> Mul<Element<F>> for Element<F> {
    type Output = Element<F>;

    fn mul(self, rhs: Element<F>) -> Element<F> {
        &self * &rhs
    }
}

impl<F: Field> Mul<&Element<F>> for Element<F> {
    type Output = Element<F>;

    fn mul(self, rhs: &Element<F>) -> Element<F> {
        &self * rhs
    }
}

impl<F: Field> Mul<Element<F>> for &Element<F> {
    type Output = Element<F>;

    fn mul(self, rhs: Element<F>) -> Element<F> {
        self * &rhs
    }
}

impl<F: Field> Mul<&Element<F>> for &Element<F> {
    type Output = Element<F>;

    fn mul(self, rhs: &Element<F>) -> Element<F> {
        Element::from((self.to_biguint() * rhs.to_biguint()) % F::order())
    }
}

impl<F: Field> Mul<u128> for Element<F> {
    type Output = Element<F>;

    fn mul(self, rhs: u128) -> Element<F> {
        &self * rhs
    }
}

impl<F: Field> Mul<u128> for &Element<F> {
    type Output = Element<F>;

    fn mul(self, rhs: u128) -> Element<F> {
        self * Element::from(rhs)
    }
}

impl<F: Field> MulAssign for Element<F> {
    fn mul_assign(&mut self, rhs: Element<F>) {
        *self *= &rhs;
    }
}

impl<F: Field> MulAssign<&Element<F>> for Element<F> {
    fn mul_assign(&mut self, rhs: &Element<F>) {
        *self = self.clone() * rhs;
    }
}

impl<F: Field> MulAssign<u128> for Element<F> {
    fn mul_assign(&mut self, rhs: u128) {
        *self = self.clone() * rhs;
    }
}

impl<F: Field> Div<Element<F>> for Element<F> {
    type Output = Element<F>;

    fn div(self, rhs: Element<F>) -> Element<F> {
        &self / &rhs
    }
}

impl<F: Field> Div<&Element<F>> for Element<F> {
    type Output = Element<F>;

    fn div(self, rhs: &Element<F>) -> Element<F> {
        &self / rhs
    }
}

impl<F: Field> Div<Element<F>> for &Element<F> {
    type Output = Element<F>;

    fn div(self, rhs: Element<F>) -> Element<F> {
        self / &rhs
    }
}

impl<F: Field> Div<&Element<F>> for &Element<F> {
    type Output = Element<F>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: &Element<F>) -> Element<F> {
        self * rhs.multiplicative_inverse()
    }
}

impl<F: Field> Div<u128> for Element<F> {
    type Output = Element<F>;

    fn div(self, rhs: u128) -> Element<F> {
        &self / rhs
    }
}

impl<F: Field> Div<u128> for &Element<F> {
    type Output = Element<F>;

    fn div(self, rhs: u128) -> Element<F> {
        self / Element::from(rhs)
    }
}

impl<F: Field> DivAssign for Element<F> {
    fn div_assign(&mut self, rhs: Element<F>) {
        *self /= &rhs;
    }
}

impl<F: Field> DivAssign<&Element<F>> for Element<F> {
    fn div_assign(&mut self, rhs: &Element<F>) {
        *self = self.clone() / rhs;
    }
}

impl<F: Field> DivAssign<u128> for Element<F> {
    fn div_assign(&mut self, rhs: u128) {
        *self = self.clone() / rhs;
    }
}

impl<F: Field> Shl<usize> for Element<F> {
    type Output = Element<F>;

    fn shl(self, rhs: usize) -> Element<F> {
        &self << rhs
    }
}

impl<F: Field> Shl<usize> for &Element<F> {
    type Output = Element<F>;

    fn shl(self, rhs: usize) -> Element<F> {
        Element::from(self.to_biguint() << rhs)
    }
}

impl<F: Field> fmt::Display for Element<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_biguint())
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use itertools::assert_equal;

    use crate::field::Element;
    use crate::test_util::{F257, F7};

    #[test]
    fn addition() {
        type F = F257;

        assert_eq!(
            Element::<F>::from(2u8),
            Element::one() + Element::one());

        assert_eq!(
            Element::<F>::from(33u8),
            Element::from(13u8) + Element::from(20u8));
    }

    #[test]
    fn addition_overflow() {
        type F = F7;

        assert_eq!(
            Element::<F>::from(3u8),
            Element::from(5u8) + Element::from(5u8));
    }

    #[test]
    fn additive_inverse() {
        type F = F7;

        assert_eq!(
            Element::<F>::from(6u8),
            -Element::one());

        assert_eq!(
            Element::<F>::zero(),
            Element::from(5u8) + -Element::from(5u8));
    }

    #[test]
    fn multiplicative_inverse() {
        type F = F7;

        // Verified with a bit of Python code:
        // >>> f = 7
        // >>> [[y for y in range(f) if x * y % f == 1] for x in range(f)]
        // [[], [1], [4], [5], [2], [3], [6]]
        assert_eq!(Element::<F>::from(0u8), Element::from(0u8).multiplicative_inverse_or_zero());
        assert_eq!(Element::<F>::from(1u8), Element::from(1u8).multiplicative_inverse_or_zero());
        assert_eq!(Element::<F>::from(4u8), Element::from(2u8).multiplicative_inverse_or_zero());
        assert_eq!(Element::<F>::from(5u8), Element::from(3u8).multiplicative_inverse_or_zero());
        assert_eq!(Element::<F>::from(2u8), Element::from(4u8).multiplicative_inverse_or_zero());
        assert_eq!(Element::<F>::from(3u8), Element::from(5u8).multiplicative_inverse_or_zero());
        assert_eq!(Element::<F>::from(6u8), Element::from(6u8).multiplicative_inverse_or_zero());
    }

    #[test]
    fn multiplication_overflow() {
        type F = F7;

        assert_eq!(
            Element::<F>::from(2u8),
            Element::from(3u8) * Element::from(3u8));
    }

    #[test]
    fn bits_0() {
        let x = Element::<F257>::zero();
        let n: usize = 20;
        assert_equal(
            iter::repeat(false).take(n),
            (0..n).map(|i| x.bit(i)));
    }

    #[test]
    fn bits_19() {
        let x = Element::<F257>::from(19u8);
        assert_eq!(true, x.bit(0));
        assert_eq!(true, x.bit(1));
        assert_eq!(false, x.bit(2));
        assert_eq!(false, x.bit(3));
        assert_eq!(true, x.bit(4));
        assert_eq!(false, x.bit(5));
        assert_eq!(false, x.bit(6));
        assert_eq!(false, x.bit(7));
        assert_eq!(false, x.bit(8));
        assert_eq!(false, x.bit(9));
    }

    #[test]
    fn order_of_elements() {
        type F = F257;
        for i in 0u8..50 {
            assert!(Element::<F>::from(i) < Element::<F>::from(i + 1));
        }
    }
}

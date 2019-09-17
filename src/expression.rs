use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use itertools::Itertools;
use num::BigUint;
use num_traits::One;
use num_traits::Zero;

use crate::field::{Element, Field};
use crate::wire::{BinaryWire, BooleanWire, Wire};
use crate::wire_values::WireValues;

/// A linear combination of wires.
#[derive(Debug, Eq, PartialEq)]
pub struct Expression<F: Field> {
    /// The coefficient of each wire. Wires with a coefficient of zero are omitted.
    coefficients: BTreeMap<Wire, Element<F>>,
}

impl<F: Field> Expression<F> {
    /// Creates a new expression with the given wire coefficients.
    pub fn new(coefficients: BTreeMap<Wire, Element<F>>) -> Self {
        let nonzero_coefficients = coefficients.into_iter()
            .filter(|(_k, v)| v.is_nonzero())
            .collect();
        Expression { coefficients: nonzero_coefficients }
    }

    pub fn coefficients(&self) -> &BTreeMap<Wire, Element<F>> {
        &self.coefficients
    }

    /// The sum of zero or more wires, each with an implied coefficient of 1.
    pub fn sum_of_wires(wires: &[Wire]) -> Self {
        Expression {
            coefficients: wires.iter()
                .map(|&v| (v, Element::one()))
                .collect()
        }
    }

    /// The collectivization of all existing Expression’s Wires with each destination Wire’s
    /// coefficient the sum of each source’s coefficients.
    pub fn sum_of_expressions(expressions: &[Expression<F>]) -> Self {
        let mut merged_coefficients = BTreeMap::new();
        for exp in expressions {
            for (&wire, coefficient) in &exp.coefficients {
                *merged_coefficients.entry(wire).or_insert_with(Element::zero) += coefficient
            }
        }
        Expression::new(merged_coefficients)
    }

    pub fn zero() -> Self {
        Expression { coefficients: BTreeMap::new() }
    }

    pub fn one() -> Self {
        Expression::from(1u128)
    }

    /// The additive inverse of 1.
    pub fn neg_one() -> Self {
        -Expression::one()
    }

    pub fn num_terms(&self) -> usize {
        self.coefficients.len()
    }

    /// Return Some(c) if this is a constant c, otherwise None.
    pub fn as_constant(&self) -> Option<Element<F>> {
        if self.num_terms() == 1 {
            self.coefficients.get(&Wire::ONE).cloned()
        } else {
            None
        }
    }

    /// Return a vector of all wires that this expression depends on.
    pub fn dependencies(&self) -> Vec<Wire> {
        self.coefficients.keys().copied().collect()
    }

    pub fn evaluate(&self, wire_values: &WireValues<F>) -> Element<F> {
        self.coefficients.iter().fold(Element::zero(),
            |sum, (wire, coefficient)| sum + (wire_values.get(*wire) * coefficient))
    }
}

impl<F: Field> Clone for Expression<F> {
    fn clone(&self) -> Self {
        Expression { coefficients: self.coefficients.clone() }
    }
}

impl<F: Field> From<Wire> for Expression<F> {
    fn from(wire: Wire) -> Self {
        Expression::new(
            [(wire, Element::one())].iter().cloned().collect())
    }
}

impl<F: Field> From<&Wire> for Expression<F> {
    fn from(wire: &Wire) -> Self {
        Expression::from(*wire)
    }
}

impl<F: Field> From<Element<F>> for Expression<F> {
    fn from(value: Element<F>) -> Self {
        Expression::new(
            [(Wire::ONE, value)].iter().cloned().collect())
    }
}

impl<F: Field> From<&Element<F>> for Expression<F> {
    fn from(value: &Element<F>) -> Self {
        Expression::from(value.clone())
    }
}

impl<F: Field> From<usize> for Expression<F> {
    fn from(value: usize) -> Self {
        Expression::from(Element::from(value))
    }
}

impl<F: Field> From<u128> for Expression<F> {
    fn from(value: u128) -> Self {
        Expression::from(Element::from(value))
    }
}

impl<F: Field> From<u64> for Expression<F> {
    fn from(value: u64) -> Self {
        Expression::from(Element::from(value))
    }
}

impl<F: Field> From<u32> for Expression<F> {
    fn from(value: u32) -> Self {
        Expression::from(Element::from(value))
    }
}

impl<F: Field> From<u16> for Expression<F> {
    fn from(value: u16) -> Self {
        Expression::from(Element::from(value))
    }
}

impl<F: Field> From<u8> for Expression<F> {
    fn from(value: u8) -> Self {
        Expression::from(Element::from(value))
    }
}

impl<F: Field> From<bool> for Expression<F> {
    fn from(value: bool) -> Self {
        Expression::from(Element::from(value))
    }
}

impl<F: Field> Neg for &Expression<F> {
    type Output = Expression<F>;

    fn neg(self) -> Expression<F> {
        self * -Element::one()
    }
}

impl<F: Field> Neg for Expression<F> {
    type Output = Expression<F>;

    fn neg(self) -> Expression<F> {
        -&self
    }
}

impl<F: Field> Add<Expression<F>> for Expression<F> {
    type Output = Expression<F>;

    fn add(self, rhs: Expression<F>) -> Expression<F> {
        &self + &rhs
    }
}

impl<F: Field> Add<&Expression<F>> for Expression<F> {
    type Output = Expression<F>;

    fn add(self, rhs: &Expression<F>) -> Expression<F> {
        &self + rhs
    }
}

impl<F: Field> Add<Expression<F>> for &Expression<F> {
    type Output = Expression<F>;

    fn add(self, rhs: Expression<F>) -> Expression<F> {
        self + &rhs
    }
}

impl<F: Field> Add<&Expression<F>> for &Expression<F> {
    type Output = Expression<F>;

    fn add(self, rhs: &Expression<F>) -> Expression<F> {
        // TODO: Use Expression::sum_of_expressions
        let mut merged_coefficients = self.coefficients.clone();
        for (wire, coefficient) in rhs.coefficients.clone() {
            *merged_coefficients.entry(wire).or_insert_with(Element::zero) += coefficient
        }
        Expression::new(merged_coefficients)
    }
}

impl<F: Field> AddAssign for Expression<F> {
    fn add_assign(&mut self, rhs: Expression<F>) {
        *self += &rhs;
    }
}

impl<F: Field> AddAssign<&Expression<F>> for Expression<F> {
    fn add_assign(&mut self, rhs: &Expression<F>) {
        // TODO: Merge coefficients instead.
        *self = self.clone() + rhs;
    }
}

impl<F: Field> Sub<Expression<F>> for Expression<F> {
    type Output = Expression<F>;

    fn sub(self, rhs: Expression<F>) -> Self::Output {
        &self - &rhs
    }
}

impl<F: Field> Sub<&Expression<F>> for Expression<F> {
    type Output = Expression<F>;

    fn sub(self, rhs: &Expression<F>) -> Self::Output {
        &self - rhs
    }
}

impl<F: Field> Sub<Expression<F>> for &Expression<F> {
    type Output = Expression<F>;

    fn sub(self, rhs: Expression<F>) -> Self::Output {
        self - &rhs
    }
}

impl<F: Field> Sub<&Expression<F>> for &Expression<F> {
    type Output = Expression<F>;

    fn sub(self, rhs: &Expression<F>) -> Self::Output {
        self + -rhs
    }
}

impl<F: Field> SubAssign for Expression<F> {
    fn sub_assign(&mut self, rhs: Expression<F>) {
        *self -= &rhs;
    }
}

impl<F: Field> SubAssign<&Expression<F>> for Expression<F> {
    fn sub_assign(&mut self, rhs: &Expression<F>) {
        *self = &*self - rhs;
    }
}

impl<F: Field> Mul<Element<F>> for Expression<F> {
    type Output = Expression<F>;

    fn mul(self, rhs: Element<F>) -> Expression<F> {
        &self * &rhs
    }
}

impl<F: Field> Mul<&Element<F>> for Expression<F> {
    type Output = Expression<F>;

    fn mul(self, rhs: &Element<F>) -> Expression<F> {
        &self * rhs
    }
}

impl<F: Field> Mul<Element<F>> for &Expression<F> {
    type Output = Expression<F>;

    fn mul(self, rhs: Element<F>) -> Expression<F> {
        self * &rhs
    }
}

impl<F: Field> Mul<&Element<F>> for &Expression<F> {
    type Output = Expression<F>;

    fn mul(self, rhs: &Element<F>) -> Expression<F> {
        Expression::new(
            self.coefficients.iter()
                .map(|(k, v)| (*k, v * rhs))
                .collect())
    }
}

impl<F: Field> Mul<u128> for Expression<F> {
    type Output = Expression<F>;

    fn mul(self, rhs: u128) -> Expression<F> {
        &self * rhs
    }
}

impl<F: Field> Mul<u128> for &Expression<F> {
    type Output = Expression<F>;

    fn mul(self, rhs: u128) -> Expression<F> {
        Expression::new(
            self.coefficients.iter()
                .map(|(k, v)| (*k, v * rhs))
                .collect())
    }
}

impl<F: Field> MulAssign<Element<F>> for Expression<F> {
    fn mul_assign(&mut self, rhs: Element<F>) {
        *self *= &rhs;
    }
}

impl<F: Field> MulAssign<&Element<F>> for Expression<F> {
    fn mul_assign(&mut self, rhs: &Element<F>) {
        *self = self.clone() * rhs;
    }
}

impl<F: Field> MulAssign<u128> for Expression<F> {
    fn mul_assign(&mut self, rhs: u128) {
        *self = self.clone() * rhs;
    }
}

impl<F: Field> Div<Element<F>> for Expression<F> {
    type Output = Expression<F>;

    fn div(self, rhs: Element<F>) -> Expression<F> {
        &self / &rhs
    }
}

impl<F: Field> Div<&Element<F>> for Expression<F> {
    type Output = Expression<F>;

    fn div(self, rhs: &Element<F>) -> Expression<F> {
        &self / rhs
    }
}

impl<F: Field> Div<Element<F>> for &Expression<F> {
    type Output = Expression<F>;

    fn div(self, rhs: Element<F>) -> Expression<F> {
        self / &rhs
    }
}

impl<F: Field> Div<&Element<F>> for &Expression<F> {
    type Output = Expression<F>;

    fn div(self, rhs: &Element<F>) -> Expression<F> {
        Expression::new(
            self.coefficients.iter()
                .map(|(k, v)| (*k, v / rhs))
                .collect())
    }
}

impl<F: Field> Div<u128> for Expression<F> {
    type Output = Expression<F>;

    fn div(self, rhs: u128) -> Expression<F> {
        &self / rhs
    }
}

impl<F: Field> Div<u128> for &Expression<F> {
    type Output = Expression<F>;

    fn div(self, rhs: u128) -> Expression<F> {
        Expression::new(
            self.coefficients.iter()
                .map(|(k, v)| (*k, v / rhs))
                .collect())
    }
}

impl<F: Field> DivAssign<Element<F>> for Expression<F> {
    fn div_assign(&mut self, rhs: Element<F>) {
        *self /= &rhs;
    }
}

impl<F: Field> DivAssign<&Element<F>> for Expression<F> {
    fn div_assign(&mut self, rhs: &Element<F>) {
        let self_immutable: &Expression<F> = self;
        *self = self_immutable / rhs;
    }
}

impl<F: Field> DivAssign<u128> for Expression<F> {
    fn div_assign(&mut self, rhs: u128) {
        let self_immutable: &Expression<F> = self;
        *self = self_immutable / rhs;
    }
}

impl<F: Field> fmt::Display for Expression<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let term_strings: Vec<String> = self.coefficients.iter()
            .sorted_by(|(k1, _v1), (k2, _v2)| k1.cmp(k2))
            .map(|(k, v)| {
                if *k == Wire::ONE {
                    format!("{}", v)
                } else if v.is_one() {
                    format!("{}", k)
                } else {
                    format!("{} * {}", k, v)
                }
            })
            .collect();
        let s = if term_strings.is_empty() {
            "0".to_string()
        } else {
            term_strings.join(" + ")
        };
        write!(f, "{}", s)
    }
}


/// An `Expression` whose value is known to be binary.
#[derive(Debug)]
pub struct BooleanExpression<F: Field> {
    expression: Expression<F>,
}

impl<F: Field> BooleanExpression<F> {
    /// Create a new bit. This is unsafe in the sense that a "equals 0 or 1" assertion will not be
    /// added automatically. You can use this method if you already know that a quantity will equal
    /// 0 or 1, for example if you computed 1 - b where b is a Bit.
    pub fn new_unsafe(expression: Expression<F>) -> Self {
        BooleanExpression { expression }
    }

    pub fn _false() -> Self {
        Self::from(false)
    }

    pub fn _true() -> Self {
        Self::from(true)
    }

    pub fn expression(&self) -> &Expression<F> {
        &self.expression
    }

    pub fn dependencies(&self) -> Vec<Wire> {
        self.expression.dependencies()
    }

    pub fn evaluate(&self, values: &WireValues<F>) -> bool {
        match self.expression.evaluate(values) {
            ref x if x.is_zero() => false,
            ref x if x.is_one() => true,
            _ => panic!("Boolean expression did not evaluate to [0, 1]")
        }
    }
}

impl<F: Field> Clone for BooleanExpression<F> {
    fn clone(&self) -> Self {
        BooleanExpression { expression: self.expression.clone() }
    }
}

impl<F: Field> From<&BooleanWire> for BooleanExpression<F> {
    fn from(wire: &BooleanWire) -> Self {
        BooleanExpression::new_unsafe(wire.wire.into())
    }
}

impl<F: Field> From<BooleanWire> for BooleanExpression<F> {
    fn from(wire: BooleanWire) -> Self {
        BooleanExpression::from(&wire)
    }
}

impl<F: Field> From<bool> for BooleanExpression<F> {
    fn from(b: bool) -> Self {
        BooleanExpression::new_unsafe(b.into())
    }
}

/// A `BinaryExpression` is comprised of several bits, each one being a `BooleanExpression`.
///
/// The sequence of bits is often interpreted as an unsigned integer for the purpose of binary
/// arithmetic. When doing so, our convention is to treat the left-most bit as the least
/// significant, and the right-most bit as the most significant.
#[derive(Debug)]
pub struct BinaryExpression<F: Field> {
    /// The sequence of bits, ordered from least significant to most significant.
    pub bits: Vec<BooleanExpression<F>>,
}

#[allow(clippy::len_without_is_empty)]
impl<F: Field> BinaryExpression<F> {
    /// The number of bits.
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn zero() -> Self {
        BinaryExpression { bits: Vec::new() }
    }

    /// Truncate the bits in this expression, discarding the more significant bits while keeping the
    /// less significant bits.
    pub fn truncate(&mut self, l: usize) {
        assert!(l <= self.len());
        self.bits.truncate(l);
    }

    /// Return a copy of this expression truncated to `l` bits, discarding the more significant bits
    /// while keeping the less significant bits.
    pub fn truncated(&self, l: usize) -> Self {
        let mut result = self.clone();
        result.truncate(l);
        result
    }

    /// Pad this bit vector, adding 0 bits on the more significant side.
    pub fn pad(&mut self, l: usize) {
        assert!(l >= self.len());
        while self.bits.len() < l {
            self.bits.push(BooleanExpression::_false());
        }
    }

    /// Return a copy this bit vector, with 0 bits added on the more significant side.
    pub fn padded(&self, l: usize) -> Self {
        let mut result = self.clone();
        result.pad(l);
        result
    }

    pub fn chunks(&self, chunk_bits: usize) -> Vec<BinaryExpression<F>> {
        self.bits.chunks(chunk_bits)
            .map(|chunk| BinaryExpression { bits: chunk.to_vec() })
            .collect()
    }

    pub fn add_most_significant(&mut self, bit: BooleanExpression<F>) {
        self.bits.push(bit);
    }

    /// Join these bits into the field element they encode. This method requires that
    /// `2^self.len() < |F|`, otherwise the result might not fit in a single field element.
    pub fn join(&self) -> Expression<F> {
        assert!(BigUint::one() << self.len() <= F::order(),
                "Binary expression is too large to fit in a single field element");
        self.join_allowing_overflow()
    }

    /// Join these bits into the field element they encode. This method allows binary expressions of
    /// any size, so overflow is possible.
    pub fn join_allowing_overflow(&self) -> Expression<F> {
        self.bits.iter().enumerate().fold(Expression::zero(),
            |sum, (i, bit)| sum + (&bit.expression * (Element::one() << i)))
    }

    pub fn dependencies(&self) -> Vec<Wire> {
        let mut all = HashSet::new();
        for bool_expression in self.bits.iter() {
            all.extend(bool_expression.dependencies());
        }
        all.into_iter().collect()
    }

    pub fn evaluate(&self, values: &WireValues<F>) -> BigUint {
        self.bits.iter().enumerate().fold(BigUint::zero(),
            |sum, (i, bit)| if bit.evaluate(values) { sum + (BigUint::one() << i) } else { sum } )
    }

    pub fn concat(expressions: &[BinaryExpression<F>]) -> Self {
        let bits = expressions.iter().map(|exp| exp.bits.clone()).concat();
        BinaryExpression { bits }
    }
}

impl<F: Field> Clone for BinaryExpression<F> {
    fn clone(&self) -> Self {
        BinaryExpression { bits: self.bits.clone() }
    }
}

impl<F: Field> From<&BinaryWire> for BinaryExpression<F> {
    fn from(wire: &BinaryWire) -> Self {
        BinaryExpression {
            bits: wire.bits.iter()
                .map(BooleanExpression::from)
                .collect()
        }
    }
}

impl<F: Field> From<BinaryWire> for BinaryExpression<F> {
    fn from(wire: BinaryWire) -> Self {
        BinaryExpression::from(&wire)
    }
}

impl<F: Field> From<BigUint> for BinaryExpression<F> {
    fn from(value: BigUint) -> Self {
        let n = value.bits();
        let bits = (0..n).map(|i| {
            let b = ((&value >> i) & BigUint::one()).is_one();
            BooleanExpression::from(b)
        }).collect();
        BinaryExpression { bits }
    }
}

impl<F: Field> From<usize> for BinaryExpression<F> {
    fn from(value: usize) -> Self {
        Self::from(BigUint::from(value))
    }
}

impl<F: Field> From<u128> for BinaryExpression<F> {
    fn from(value: u128) -> Self {
        Self::from(BigUint::from(value))
    }
}

impl<F: Field> From<u64> for BinaryExpression<F> {
    fn from(value: u64) -> Self {
        Self::from(BigUint::from(value))
    }
}

impl<F: Field> From<u32> for BinaryExpression<F> {
    fn from(value: u32) -> Self {
        Self::from(BigUint::from(value))
    }
}

impl<F: Field> From<u16> for BinaryExpression<F> {
    fn from(value: u16) -> Self {
        Self::from(BigUint::from(value))
    }
}

impl<F: Field> From<u8> for BinaryExpression<F> {
    fn from(value: u8) -> Self {
        Self::from(BigUint::from(value))
    }
}

#[cfg(test)]
mod tests {
    use crate::{GadgetBuilder, BinaryExpression};
    use crate::test_util::F257;

    #[test]
    fn join_fermat_prime_field() {
        // Test joining a binary expression into a field element, where the (Fermat prime) field is
        // just large enough to fit the expression.
        let mut builder = GadgetBuilder::<F257>::new();
        let wire = builder.binary_wire(8);
        let exp = BinaryExpression::<F257>::from(&wire);
        exp.join();
    }

    #[test]
    #[should_panic]
    fn join_fermat_prime_field_overflow() {
        // Test joining a binary expression into a field element, where the (Fermat prime) field is
        // too small to fit the expression.
        let mut builder = GadgetBuilder::<F257>::new();
        let wire = builder.binary_wire(9);
        let exp = BinaryExpression::<F257>::from(&wire);
        exp.join();
    }
}
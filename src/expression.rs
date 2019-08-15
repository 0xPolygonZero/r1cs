use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

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
    coefficients: HashMap<Wire, Element<F>>,
}

impl<F: Field> Expression<F> {
    pub fn new(coefficients: HashMap<Wire, Element<F>>) -> Self {
        let nonzero_coefficients = coefficients.into_iter()
            .filter(|(_k, v)| *v != Element::zero())
            .collect();
        Expression { coefficients: nonzero_coefficients }
    }

    /// The sum of zero or more wires, each with an implied coefficient of 1.
    pub fn sum(wires: &[Wire]) -> Self {
        Expression {
            coefficients: wires.iter()
                .map(|&v| (v, Element::one()))
                .collect()
        }
    }

    pub fn zero() -> Self {
        Expression { coefficients: HashMap::new() }
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
        let mut sum = Element::zero();
        for (&wire, coefficient) in &self.coefficients {
            sum += wire_values.get(wire) * coefficient;
        }
        sum
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
        Self::new_unsafe(Expression::zero())
    }

    pub fn _true() -> Self {
        Self::new_unsafe(Expression::one())
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

/// A "binary expression" which is comprised of several bits, each one being a boolean expression.
#[derive(Debug)]
pub struct BinaryExpression<F: Field> {
    /// The list of bits, ordered from least significant to most significant.
    pub bits: Vec<BooleanExpression<F>>,
}

#[allow(clippy::len_without_is_empty)]
impl<F: Field> BinaryExpression<F> {
    /// The number of bits.
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Truncate this bit vector, discarding the more significant bits while keeping the less
    /// significant bits.
    // TODO: Convert to mutable.
    pub fn truncated(&self, l: usize) -> Self {
        assert!(l <= self.len());
        let mut truncated_bits = self.bits.clone();
        truncated_bits.truncate(l);
        BinaryExpression { bits: truncated_bits }
    }

    /// Pad this bit vector, adding 0 bits on the more significant side.
    // TODO: Convert to mutable.
    pub fn padded(&self, l: usize) -> Self {
        assert!(l >= self.len());
        let mut padded_bits = self.bits.clone();
        while padded_bits.len() < l {
            padded_bits.push(BooleanExpression::_false());
        }
        BinaryExpression { bits: padded_bits }
    }

    pub fn add_most_significant(&mut self, bit: BooleanExpression<F>) {
        self.bits.push(bit);
    }

    pub fn chunks(&self, chunk_bits: usize) -> Vec<BinaryExpression<F>> {
        self.bits.chunks(chunk_bits).map(|chunk| BinaryExpression { bits: chunk.to_vec() }).collect()
    }

    /// Join these bits into the field element they encode.
    pub fn join(&self) -> Expression<F> {
        assert!(self.len() < Element::<F>::max_bits(), "Cannot fit in a single field element");
        let mut sum = Expression::zero();
        for (i, bit) in self.bits.iter().enumerate() {
            let weight = Element::one() << i;
            sum += &bit.expression * weight;
        }
        sum
    }

    pub fn dependencies(&self) -> Vec<Wire> {
        let mut all = HashSet::new();
        for bool_expression in self.bits.iter() {
            all.extend(bool_expression.dependencies());
        }
        all.into_iter().collect()
    }

    pub fn evaluate(&self, values: &WireValues<F>) -> BigUint {
        let mut sum = BigUint::zero();
        for (i, bit) in self.bits.iter().enumerate() {
            if bit.evaluate(values) {
                sum += BigUint::one() << i;
            }
        }
        sum
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
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use itertools::Itertools;
use num::BigUint;
use num_traits::One;
use num_traits::Zero;

use crate::field_element::FieldElement;
use crate::wire::{BinaryWire, BooleanWire, Wire};
use crate::wire_values::WireValues;

/// A linear combination of wires.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    /// The coefficient of each wire. Wires with a coefficient of zero are omitted.
    coefficients: HashMap<Wire, FieldElement>,
}

impl Expression {
    pub fn new(coefficients: HashMap<Wire, FieldElement>) -> Self {
        let nonzero_coefficients = coefficients.into_iter()
            .filter(|(_k, v)| *v != FieldElement::zero())
            .collect();
        Expression { coefficients: nonzero_coefficients }
    }

    /// The sum of zero or more wires, each with an implied coefficient of 1.
    pub fn sum<'a, T>(wires: T) -> Self
        where T: IntoIterator<Item=&'a Wire> {
        Expression {
            coefficients: wires.into_iter()
                .map(|&v| (v, FieldElement::one()))
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
    pub fn as_constant(&self) -> Option<FieldElement> {
        if self.num_terms() == 1 {
            self.coefficients.get(&Wire::ONE).map(|c| c.clone())
        } else {
            None
        }
    }

    /// Return a vector of all wires that this expression depends on.
    pub fn dependencies(&self) -> Vec<Wire> {
        self.coefficients.keys()
            .map(|w| *w)
            .collect()
    }

    pub fn evaluate(&self, wire_values: &WireValues) -> FieldElement {
        let mut sum = FieldElement::zero();
        for (wire, coefficient) in &self.coefficients {
            sum += wire_values.get(wire) * coefficient;
        }
        sum
    }
}

impl From<Wire> for Expression {
    fn from(wire: Wire) -> Self {
        Expression::new(
            [(wire, FieldElement::one())].iter().cloned().collect())
    }
}

impl From<&Wire> for Expression {
    fn from(wire: &Wire) -> Self {
        Expression::from(*wire)
    }
}

impl From<FieldElement> for Expression {
    fn from(value: FieldElement) -> Self {
        Expression::new(
            [(Wire::ONE, value)].iter().cloned().collect())
    }
}

impl From<&FieldElement> for Expression {
    fn from(value: &FieldElement) -> Self {
        Expression::from(value.clone())
    }
}

impl From<u128> for Expression {
    fn from(value: u128) -> Self {
        Expression::from(FieldElement::from(value))
    }
}

impl Neg for &Expression {
    type Output = Expression;

    fn neg(self) -> Expression {
        self * -FieldElement::one()
    }
}

impl Neg for Expression {
    type Output = Expression;

    fn neg(self) -> Expression {
        -&self
    }
}

impl Add<Expression> for Expression {
    type Output = Expression;

    fn add(self, rhs: Expression) -> Expression {
        &self + &rhs
    }
}

impl Add<&Expression> for Expression {
    type Output = Expression;

    fn add(self, rhs: &Expression) -> Expression {
        &self + rhs
    }
}

impl Add<Expression> for &Expression {
    type Output = Expression;

    fn add(self, rhs: Expression) -> Expression {
        self + &rhs
    }
}

impl Add<&Expression> for &Expression {
    type Output = Expression;

    fn add(self, rhs: &Expression) -> Expression {
        let mut merged_coefficients = self.coefficients.clone();
        for (wire, coefficient) in rhs.coefficients.clone() {
            *merged_coefficients.entry(wire).or_insert(FieldElement::zero()) += coefficient
        }
        Expression::new(merged_coefficients)
    }
}

impl AddAssign for Expression {
    fn add_assign(&mut self, rhs: Expression) {
        *self += &rhs;
    }
}

impl AddAssign<&Expression> for Expression {
    fn add_assign(&mut self, rhs: &Expression) {
        *self = self.clone() + rhs;
    }
}

impl Sub<Expression> for Expression {
    type Output = Expression;

    fn sub(self, rhs: Expression) -> Self::Output {
        &self - &rhs
    }
}

impl Sub<&Expression> for Expression {
    type Output = Expression;

    fn sub(self, rhs: &Expression) -> Self::Output {
        &self - rhs
    }
}

impl Sub<Expression> for &Expression {
    type Output = Expression;

    fn sub(self, rhs: Expression) -> Self::Output {
        self - &rhs
    }
}

impl Sub<&Expression> for &Expression {
    type Output = Expression;

    fn sub(self, rhs: &Expression) -> Self::Output {
        self + -rhs
    }
}

impl SubAssign for Expression {
    fn sub_assign(&mut self, rhs: Expression) {
        *self -= &rhs;
    }
}

impl SubAssign<&Expression> for Expression {
    fn sub_assign(&mut self, rhs: &Expression) {
        *self = &*self - rhs;
    }
}

impl Mul<FieldElement> for Expression {
    type Output = Expression;

    fn mul(self, rhs: FieldElement) -> Expression {
        &self * &rhs
    }
}

impl Mul<&FieldElement> for Expression {
    type Output = Expression;

    fn mul(self, rhs: &FieldElement) -> Expression {
        &self * rhs
    }
}

impl Mul<FieldElement> for &Expression {
    type Output = Expression;

    fn mul(self, rhs: FieldElement) -> Expression {
        self * &rhs
    }
}

impl Mul<&FieldElement> for &Expression {
    type Output = Expression;

    fn mul(self, rhs: &FieldElement) -> Expression {
        Expression::new(
            self.coefficients.iter()
                .map(|(k, v)| (k.clone(), v * rhs))
                .collect())
    }
}

impl Mul<u128> for Expression {
    type Output = Expression;

    fn mul(self, rhs: u128) -> Expression {
        &self * rhs
    }
}

impl Mul<u128> for &Expression {
    type Output = Expression;

    fn mul(self, rhs: u128) -> Expression {
        Expression::new(
            self.coefficients.iter()
                .map(|(k, v)| (k.clone(), v * rhs))
                .collect())
    }
}

impl MulAssign<FieldElement> for Expression {
    fn mul_assign(&mut self, rhs: FieldElement) {
        *self *= &rhs;
    }
}

impl MulAssign<&FieldElement> for Expression {
    fn mul_assign(&mut self, rhs: &FieldElement) {
        *self = self.clone() * rhs;
    }
}

impl MulAssign<u128> for Expression {
    fn mul_assign(&mut self, rhs: u128) {
        *self = self.clone() * rhs;
    }
}

impl fmt::Display for Expression {
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
#[derive(Clone, Debug)]
pub struct BooleanExpression {
    expression: Expression,
}

impl BooleanExpression {
    /// Create a new bit. This is unsafe in the sense that a "equals 0 or 1" assertion will not be
    /// added automatically. You can use this method if you already know that a quantity will equal
    /// 0 or 1, for example if you computed 1 - b where b is a Bit.
    pub fn new_unsafe(expression: Expression) -> Self {
        BooleanExpression { expression }
    }

    pub fn _false() -> Self {
        BooleanExpression::new_unsafe(Expression::zero())
    }

    pub fn _true() -> Self {
        BooleanExpression::new_unsafe(Expression::one())
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn dependencies(&self) -> Vec<Wire> {
        self.expression.dependencies()
    }

    pub fn evaluate(&self, values: &WireValues) -> bool {
        match self.expression.evaluate(values) {
            ref x if x.is_zero() => false,
            ref x if x.is_one() => true,
            _ => panic!("Boolean expression did not evaluate to [0, 1]")
        }
    }
}

impl From<&BooleanWire> for BooleanExpression {
    fn from(wire: &BooleanWire) -> Self {
        BooleanExpression::new_unsafe(wire.wire.into())
    }
}

impl From<BooleanWire> for BooleanExpression {
    fn from(wire: BooleanWire) -> Self {
        BooleanExpression::from(&wire)
    }
}

/// A "binary expression" which is comprised of several bits, each one being a boolean expression.
#[derive(Clone, Debug)]
pub struct BinaryExpression {
    /// The list of bits, ordered from least significant to most significant.
    pub bits: Vec<BooleanExpression>,
}

impl BinaryExpression {
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

    pub fn add_most_significant(&mut self, bit: BooleanExpression) {
        self.bits.push(bit);
    }

    pub fn chunks(&self, chunk_bits: usize) -> Vec<BinaryExpression> {
        self.bits.chunks(chunk_bits).map(|chunk| BinaryExpression { bits: chunk.to_vec() }).collect()
    }

    /// Join these bits into the field element they encode.
    pub fn join(&self) -> Expression {
        assert!(self.len() < FieldElement::max_bits(), "Cannot fit in a single field element");
        let mut sum = Expression::zero();
        for (i, bit) in self.bits.iter().enumerate() {
            let weight = FieldElement::one() << i;
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

    pub fn evaluate(&self, values: &WireValues) -> BigUint {
        let mut sum = BigUint::zero();
        for (i, bit) in self.bits.iter().enumerate() {
            if bit.evaluate(values) {
                sum += BigUint::one() << i;
            }
        }
        sum
    }
}

impl From<&BinaryWire> for BinaryExpression {
    fn from(wire: &BinaryWire) -> Self {
        BinaryExpression {
            bits: wire.bits.iter()
                .map(|bool_wire| BooleanExpression::from(bool_wire))
                .collect()
        }
    }
}

impl From<BinaryWire> for BinaryExpression {
    fn from(wire: BinaryWire) -> Self {
        BinaryExpression::from(&wire)
    }
}
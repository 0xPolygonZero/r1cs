use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use itertools::Itertools;

use crate::field_element::FieldElement;
use crate::wire::Wire;
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
            .map(|w| w.clone())
            .collect()
    }

    pub fn evaluate(&self, wire_values: &WireValues) -> FieldElement {
        let mut sum = FieldElement::zero();
        for (wire, coefficient) in &self.coefficients {
            sum += wire_values.get(wire) * coefficient.clone();
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

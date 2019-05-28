use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use field_element::FieldElement;
use wire::Wire;
use wire_values::WireValues;
use itertools::Itertools;

/// A linear combination of wires.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearCombination {
    coefficients: HashMap<Wire, FieldElement>,
}

impl LinearCombination {
    pub fn new(coefficients: HashMap<Wire, FieldElement>) -> Self {
        let nonzero_coefficients = coefficients.into_iter()
            .filter(|(_k, v)| *v != FieldElement::zero())
            .collect();
        LinearCombination { coefficients: nonzero_coefficients }
    }

    /// The sum of zero or more wires, each with an implied coefficient of 1.
    pub fn sum<'a, T>(wires: T) -> Self
        where T: IntoIterator<Item=&'a Wire> {
        LinearCombination {
            coefficients: wires.into_iter()
                .map(|&v| (v, FieldElement::one()))
                .collect()
        }
    }

    /// Join a vector of bit wires into the field element it encodes.
    pub fn join_bits<'a, T>(bit_wires: T) -> Self
        where T: IntoIterator<Item=&'a Wire> {
        LinearCombination {
            coefficients: bit_wires.into_iter().enumerate()
                .map(|(i, w)| (w.clone(), FieldElement::one() << i))
                .collect()
        }
    }

    pub fn zero() -> Self {
        LinearCombination::from(0u128)
    }

    pub fn one() -> Self {
        LinearCombination::from(1u128)
    }

    /// The additive inverse of 1.
    pub fn neg_one() -> Self {
        -LinearCombination::one()
    }

    pub fn num_terms(&self) -> usize {
        self.coefficients.len()
    }

    /// Return a vector of all wires involved in this linear combination.
    pub fn wires(&self) -> Vec<Wire> {
        self.coefficients.keys()
            .map(|w| w.clone())
            .collect()
    }

    /// Return a vector of all wires involved in this linear combination, except for the special 1
    /// wire.
    pub fn variable_wires(&self) -> Vec<Wire> {
        return self.wires().into_iter()
            .filter(|&w| w != Wire::ONE)
            .collect();
    }

    pub fn evaluate(&self, wire_values: &WireValues) -> FieldElement {
        let mut sum = FieldElement::zero();
        for (wire, coefficient) in &self.coefficients {
            sum += wire_values.get(wire) * coefficient.clone();
        }
        sum
    }
}

impl From<Wire> for LinearCombination {
    fn from(wire: Wire) -> Self {
        LinearCombination::new(
            [(wire, FieldElement::one())].iter().cloned().collect())
    }
}

impl From<FieldElement> for LinearCombination {
    fn from(value: FieldElement) -> Self {
        LinearCombination::new(
            [(Wire::ONE, value)].iter().cloned().collect())
    }
}

impl From<u128> for LinearCombination {
    fn from(value: u128) -> Self {
        LinearCombination::from(FieldElement::from(value))
    }
}

impl Neg for LinearCombination {
    type Output = LinearCombination;

    fn neg(self) -> LinearCombination {
        self * -FieldElement::one()
    }
}

impl Add<LinearCombination> for LinearCombination {
    type Output = LinearCombination;

    fn add(self, rhs: LinearCombination) -> LinearCombination {
        let mut merged_coefficients = self.coefficients.clone();
        for (wire, coefficient) in rhs.coefficients {
            *merged_coefficients.entry(wire).or_insert(FieldElement::zero()) += coefficient
        }
        LinearCombination::new(merged_coefficients)
    }
}

impl AddAssign for LinearCombination {
    fn add_assign(&mut self, rhs: LinearCombination) {
        *self = self.clone() + rhs;
    }
}

impl Sub<LinearCombination> for LinearCombination {
    type Output = LinearCombination;

    fn sub(self, rhs: LinearCombination) -> Self::Output {
        self + -rhs
    }
}

impl SubAssign for LinearCombination {
    fn sub_assign(&mut self, rhs: LinearCombination) {
        *self = self.clone() - rhs;
    }
}

impl Mul<FieldElement> for LinearCombination {
    type Output = LinearCombination;

    fn mul(self, rhs: FieldElement) -> LinearCombination {
        LinearCombination::new(
            self.coefficients.into_iter()
                .map(|(k, v)| (k, v * rhs.clone()))
                .collect())
    }
}

impl Mul<u128> for LinearCombination {
    type Output = LinearCombination;

    fn mul(self, rhs: u128) -> LinearCombination {
        LinearCombination::new(
            self.coefficients.into_iter()
                .map(|(k, v)| (k, v * rhs.clone()))
                .collect())
    }
}

impl MulAssign<FieldElement> for LinearCombination {
    fn mul_assign(&mut self, rhs: FieldElement) {
        *self = self.clone() * rhs;
    }
}

impl MulAssign<u128> for LinearCombination {
    fn mul_assign(&mut self, rhs: u128) {
        *self = self.clone() * rhs;
    }
}

impl fmt::Display for LinearCombination {
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

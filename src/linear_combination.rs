use wire::Wire;
use std::collections::HashMap;
use field_element::FieldElement;
use wire_values::WireValues;
use std::ops::{Add, Mul, AddAssign, MulAssign, Neg, Sub, SubAssign};

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

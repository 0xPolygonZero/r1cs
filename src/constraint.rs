use linear_combination::LinearCombination;
use wire_values::WireValues;

/// An R1CS constraint, of the form a * b = c, where a, b, and c are linear combinations of wires.
#[derive(Clone, Debug)]
pub struct Constraint {
    pub a: LinearCombination,
    pub b: LinearCombination,
    pub c: LinearCombination,
}

impl Constraint {
    /// Create and return a constraint which forces the given quantity to equal 0 or 1.
    pub fn binary(x: &LinearCombination) -> Self {
        Constraint {
            a: x.clone(),
            b: x.clone() - 1.into(),
            c: 0.into(),
        }
    }

    pub fn evaluate(&self, wire_values: &WireValues) -> bool {
        let a_value = self.a.evaluate(wire_values);
        let b_value = self.b.evaluate(wire_values);
        let c_value = self.c.evaluate(wire_values);
        a_value * b_value == c_value
    }
}
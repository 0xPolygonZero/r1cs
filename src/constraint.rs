use linear_combination::LinearCombination;
use wire_values::WireValues;

/// An R1CS constraint, of the form a * b = c, where a, b, and c are linear combinations of wires.
#[derive(Debug)]
pub struct Constraint {
    pub a: LinearCombination,
    pub b: LinearCombination,
    pub c: LinearCombination,
}

impl Constraint {
    pub fn evaluate(&self, wire_values: &WireValues) -> bool {
        let a_value = self.a.evaluate(wire_values);
        let b_value = self.b.evaluate(wire_values);
        let c_value = self.c.evaluate(wire_values);
        a_value * b_value == c_value
    }
}
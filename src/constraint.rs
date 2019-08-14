use std::fmt;
use std::fmt::Formatter;

use crate::expression::Expression;
use crate::field::Field;
use crate::wire_values::WireValues;

/// An R1CS constraint, of the form a * b = c, where a, b, and c are linear combinations of wires.
#[derive(Clone, Debug)]
pub struct Constraint<F: Field> {
    pub a: Expression<F>,
    pub b: Expression<F>,
    pub c: Expression<F>,
}

impl<F: Field> Constraint<F> {
    pub fn evaluate(&self, wire_values: &WireValues<F>) -> bool {
        let a_value = self.a.evaluate(wire_values);
        let b_value = self.b.evaluate(wire_values);
        let c_value = self.c.evaluate(wire_values);
        a_value * b_value == c_value
    }
}

impl<F: Field> fmt::Display for Constraint<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let a_str = if self.a.num_terms() >= 2 {
            format!("({})", self.a)
        } else {
            format!("{}", self.a)
        };

        let b_str = if self.b.num_terms() >= 2 {
            format!("({})", self.b)
        } else {
            format!("{}", self.b)
        };

        write!(f, "{} * {} = {}", a_str, b_str, self.c)
    }
}
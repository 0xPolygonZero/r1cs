//! This module contains test helper functions.

use std::borrow::Borrow;

use crate::expression::BooleanExpression;
use crate::wire_values::WireValues;
use crate::field::Field;

pub fn assert_eq_true<F, T>(x: T, values: &WireValues<F>)
    where F: Field, T: Borrow<BooleanExpression<F>> {
    assert_eq!(true, x.borrow().evaluate(values));
}

pub fn assert_eq_false<F, T>(x: T, values: &WireValues<F>)
    where F: Field, T: Borrow<BooleanExpression<F>> {
    assert_eq!(false, x.borrow().evaluate(values));
}

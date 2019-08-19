//! This module extends GadgetBuilder with boolean algebra methods.

use core::borrow::Borrow;

use crate::expression::{BooleanExpression, Expression};
use crate::field::Field;
use crate::gadget_builder::GadgetBuilder;

impl<F: Field> GadgetBuilder<F> {
    /// The negation of a boolean value.
    pub fn not<BE: Borrow<BooleanExpression<F>>>(&mut self, x: BE) -> BooleanExpression<F> {
        BooleanExpression::new_unsafe(Expression::one() - x.borrow().expression())
    }

    /// The conjunction of two boolean values.
    pub fn and<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BooleanExpression<F>
        where BE1: Borrow<BooleanExpression<F>>, BE2: Borrow<BooleanExpression<F>> {
        BooleanExpression::new_unsafe(self.product(x.borrow().expression(), y.borrow().expression()))
    }

    /// The disjunction of two boolean values.
    pub fn or<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BooleanExpression<F>
        where BE1: Borrow<BooleanExpression<F>>, BE2: Borrow<BooleanExpression<F>> {
        let x_exp = x.borrow().expression();
        let y_exp = y.borrow().expression();
        BooleanExpression::new_unsafe(
            x_exp + y_exp - self.product(x_exp, y_exp))
    }

    /// The exclusive disjunction of two boolean values.
    pub fn xor<BE1, BE2>(&mut self, x: BE1, y: BE2) -> BooleanExpression<F>
        where BE1: Borrow<BooleanExpression<F>>, BE2: Borrow<BooleanExpression<F>> {
        let x_exp = x.borrow().expression();
        let y_exp = y.borrow().expression();
        BooleanExpression::new_unsafe(x_exp + y_exp - self.product(x_exp, y_exp) * 2u128)
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::BooleanExpression;
    use crate::field::Bn128;
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn and() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let (x, y) = (builder.boolean_wire(), builder.boolean_wire());
        let and = builder.and(BooleanExpression::from(x), BooleanExpression::from(y));
        let gadget = builder.build();

        let mut values00 = boolean_values!(x => false, y => false);
        assert!(gadget.execute(&mut values00));
        assert_eq!(false, and.evaluate(&values00));

        let mut values01 = boolean_values!(x => false, y => true);
        assert!(gadget.execute(&mut values01));
        assert_eq!(false, and.evaluate(&values01));

        let mut values10 = boolean_values!(x => true, y => false);
        assert!(gadget.execute(&mut values10));
        assert_eq!(false, and.evaluate(&values10));

        let mut values11 = boolean_values!(x => true, y => true);
        assert!(gadget.execute(&mut values11));
        assert_eq!(true, and.evaluate(&values11));
    }

    #[test]
    fn or() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let (x, y) = (builder.boolean_wire(), builder.boolean_wire());
        let or = builder.or(BooleanExpression::from(x), BooleanExpression::from(y));
        let gadget = builder.build();

        let mut values00 = boolean_values!(x => false, y => false);
        assert!(gadget.execute(&mut values00));
        assert_eq!(false, or.evaluate(&values00));

        let mut values01 = boolean_values!(x => false, y => true);
        assert!(gadget.execute(&mut values01));
        assert_eq!(true, or.evaluate(&values01));

        let mut values10 = boolean_values!(x => true, y => false);
        assert!(gadget.execute(&mut values10));
        assert_eq!(true, or.evaluate(&values10));

        let mut values11 = boolean_values!(x => true, y => true);
        assert!(gadget.execute(&mut values11));
        assert_eq!(true, or.evaluate(&values11));
    }

    #[test]
    fn xor() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let (x, y) = (builder.boolean_wire(), builder.boolean_wire());
        let xor = builder.xor(BooleanExpression::from(x), BooleanExpression::from(y));
        let gadget = builder.build();

        let mut values00 = boolean_values!(x => false, y => false);
        assert!(gadget.execute(&mut values00));
        assert_eq!(false, xor.evaluate(&values00));

        let mut values01 = boolean_values!(x => false, y => true);
        assert!(gadget.execute(&mut values01));
        assert_eq!(true, xor.evaluate(&values01));

        let mut values10 = boolean_values!(x => true, y => false);
        assert!(gadget.execute(&mut values10));
        assert_eq!(true, xor.evaluate(&values10));

        let mut values11 = boolean_values!(x => true, y => true);
        assert!(gadget.execute(&mut values11));
        assert_eq!(false, xor.evaluate(&values11));
    }
}
//! This module extends GadgetBuilder with boolean algebra methods.

use crate::bits::BooleanExpression;
use crate::gadget_builder::GadgetBuilder;
use crate::expression::Expression;

impl GadgetBuilder {
    /// The negation of a boolean value.
    pub fn not(&mut self, x: BooleanExpression) -> BooleanExpression {
        BooleanExpression::new_unsafe(Expression::one() - x.expression())
    }

    /// The conjunction of two boolean values.
    pub fn and(&mut self, x: BooleanExpression, y: BooleanExpression) -> BooleanExpression {
        BooleanExpression::new_unsafe(self.product(x.expression().clone(), y.expression().clone()))
    }

    /// The disjunction of two boolean values.
    pub fn or(&mut self, x: BooleanExpression, y: BooleanExpression) -> BooleanExpression {
        BooleanExpression::new_unsafe(
            x.expression()
                + y.expression()
                - self.product(x.expression().clone(), y.expression().clone()))
    }

    /// The exclusive disjunction of two boolean values.
    pub fn xor(&mut self, x: BooleanExpression, y: BooleanExpression) -> BooleanExpression {
        BooleanExpression::new_unsafe(x.expression().clone() + y.expression().clone()
            - self.product(x.expression().clone(), y.expression().clone()) * 2u128)
    }
}

#[cfg(test)]
mod tests {
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn and() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.boolean_wire(), builder.boolean_wire());
        let and = builder.and(x.into(), y.into());
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
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.boolean_wire(), builder.boolean_wire());
        let or = builder.or(x.into(), y.into());
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
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.boolean_wire(), builder.boolean_wire());
        let xor = builder.xor(x.into(), y.into());
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
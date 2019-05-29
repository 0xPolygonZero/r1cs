use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;

impl GadgetBuilder {
    /// The conjunction of two binary values. Assumes both inputs are binary, otherwise the result
    /// is undefined.
    pub fn and(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        self.product(x, y)
    }

    /// The disjunction of two binary values. Assumes both inputs are binary, otherwise the result
    /// is undefined.
    pub fn or(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        x.clone() + y.clone() - self.and(x, y)
    }

    /// The exclusive disjunction of two binary values. Assumes both inputs are binary, otherwise
    /// the result is undefined.
    pub fn xor(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        x.clone() + y.clone() - self.and(x, y) * 2u128
    }
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;
    use field_element::FieldElement;

    #[test]
    fn and() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let and = builder.and(x.into(), y.into());
        let gadget = builder.build();

        let mut values00 = wire_values!(x => 0.into(), y => 0.into());
        assert!(gadget.execute(&mut values00));
        assert_eq!(FieldElement::zero(), and.evaluate(&values00));

        let mut values01 = wire_values!(x => 0.into(), y => 1.into());
        assert!(gadget.execute(&mut values01));
        assert_eq!(FieldElement::zero(), and.evaluate(&values01));

        let mut values10 = wire_values!(x => 1.into(), y => 0.into());
        assert!(gadget.execute(&mut values10));
        assert_eq!(FieldElement::zero(), and.evaluate(&values10));

        let mut values11 = wire_values!(x => 1.into(), y => 1.into());
        assert!(gadget.execute(&mut values11));
        assert_eq!(FieldElement::one(), and.evaluate(&values11));
    }

    #[test]
    fn or() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let or = builder.or(x.into(), y.into());
        let gadget = builder.build();

        let mut values00 = wire_values!(x => 0.into(), y => 0.into());
        assert!(gadget.execute(&mut values00));
        assert_eq!(FieldElement::zero(), or.evaluate(&values00));

        let mut values01 = wire_values!(x => 0.into(), y => 1.into());
        assert!(gadget.execute(&mut values01));
        assert_eq!(FieldElement::one(), or.evaluate(&values01));

        let mut values10 = wire_values!(x => 1.into(), y => 0.into());
        assert!(gadget.execute(&mut values10));
        assert_eq!(FieldElement::one(), or.evaluate(&values10));

        let mut values11 = wire_values!(x => 1.into(), y => 1.into());
        assert!(gadget.execute(&mut values11));
        assert_eq!(FieldElement::one(), or.evaluate(&values11));
    }

    #[test]
    fn xor() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let xor = builder.xor(x.into(), y.into());
        let gadget = builder.build();

        let mut values00 = wire_values!(x => 0.into(), y => 0.into());
        assert!(gadget.execute(&mut values00));
        assert_eq!(FieldElement::zero(), xor.evaluate(&values00));

        let mut values01 = wire_values!(x => 0.into(), y => 1.into());
        assert!(gadget.execute(&mut values01));
        assert_eq!(FieldElement::one(), xor.evaluate(&values01));

        let mut values10 = wire_values!(x => 1.into(), y => 0.into());
        assert!(gadget.execute(&mut values10));
        assert_eq!(FieldElement::one(), xor.evaluate(&values10));

        let mut values11 = wire_values!(x => 1.into(), y => 1.into());
        assert!(gadget.execute(&mut values11));
        assert_eq!(FieldElement::zero(), xor.evaluate(&values11));
    }
}
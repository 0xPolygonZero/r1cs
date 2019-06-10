use std::collections::HashMap;

use num::BigUint;

use crate::gadget_builder::GadgetBuilder;
use crate::linear_combination::LinearCombination;
use crate::wire::Wire;
use crate::wire_values::WireValues;

impl GadgetBuilder {
    /// Split `x` into `bits` bit wires. Assumes `x < 2^bits`.
    pub fn split(&mut self, x: LinearCombination, bits: usize) -> Vec<Wire> {
        let bit_wires = self.wires(bits);

        {
            let x = x.clone();
            let bit_wires = bit_wires.clone();

            self.generator(
                x.wires(),
                move |values: &mut WireValues| {
                    let value = x.evaluate(values);
                    assert!(value.bits() <= bits);
                    for i in 0..bits {
                        let bit_value = value.bit(i).into();
                        values.set(bit_wires[i], bit_value);
                    }
                },
            );
        }

        // Constrain each bit wire to [0, 1].
        for wire in bit_wires.clone().into_iter() {
            self.assert_binary(wire.into());
        }

        let mut bit_weights = HashMap::new();
        for (i, &wire) in bit_wires.iter().enumerate() {
            bit_weights.insert(wire, (BigUint::from(1u64) << i).into());
        }
        let weighted_sum = LinearCombination::new(bit_weights);
        self.assert_equal(x.into(), weighted_sum);

        // TODO: Needs a comparison to verify that no overflow occurred, i.e., that the sum is less
        // than the prime field size.

        bit_wires
    }
}

#[cfg(test)]
mod tests {
    use crate::field_element::FieldElement;
    use crate::gadget_builder::GadgetBuilder;

    #[test]
    fn split_19_32() {
        let mut builder = GadgetBuilder::new();
        let wire = builder.wire();
        let bit_wires = builder.split(wire.into(), 32);
        let gadget = builder.build();

        let mut wire_values = values!(wire.clone() => 19.into());
        assert!(gadget.execute(&mut wire_values));

        let false_element: FieldElement = 0.into();
        let true_element: FieldElement = 1.into();
        assert_eq!(true_element, wire_values.get(&bit_wires[0]));
        assert_eq!(true_element, wire_values.get(&bit_wires[1]));
        assert_eq!(false_element, wire_values.get(&bit_wires[2]));
        assert_eq!(false_element, wire_values.get(&bit_wires[3]));
        assert_eq!(true_element, wire_values.get(&bit_wires[4]));
        assert_eq!(false_element, wire_values.get(&bit_wires[5]));
        assert_eq!(false_element, wire_values.get(&bit_wires[6]));
    }
}
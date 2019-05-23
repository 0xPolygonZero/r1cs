use gadget_builder::GadgetBuilder;
use wire::Wire;
use witness_generator::WitnessGenerator;
use wire_values::WireValues;
use field_element::FieldElement;
use constraint::Constraint;
use std::collections::HashMap;
use linear_combination::LinearCombination;
use num::BigUint;

fn split(builder: &mut GadgetBuilder, wire: Wire, bits: usize) -> Vec<Wire> {
    let bit_wires = builder.wires(bits);

    {
        let wire = wire.clone();
        let bit_wires = bit_wires.clone();

        builder.generator(WitnessGenerator::new(
            vec![wire],
            move |values: &mut WireValues| {
                let value = values.get(&wire);
                for i in 0..bits {
                    let bit_value = FieldElement::from(value.bit(i) as u128);
                    values.set(bit_wires[i], bit_value);
                }
            },
        ));
    }

    // Constrain each bit wire to [0, 1].
    for wire in bit_wires.clone().into_iter() {
        builder.assert_binary(wire.into());
    }

    let mut bit_weights = HashMap::new();
    for (i, &wire) in bit_wires.iter().enumerate() {
        bit_weights.insert(wire, (BigUint::from(1u64) << i).into());
    }
    let weighted_sum = LinearCombination::new(bit_weights);
    builder.assert_equal(wire.into(), weighted_sum);

    bit_wires
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;
    use gadgets::split::split;
    use wire_values::WireValues;
    use field_element::FieldElement;

    #[test]
    fn split_19_32() {
        let mut builder = GadgetBuilder::new();
        let wire = builder.wire();
        let bit_wires = split(&mut builder, wire, 32);
        let gadget = builder.build();

        let mut wire_values = WireValues::new();
        wire_values.set(wire.clone(), 19.into());
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
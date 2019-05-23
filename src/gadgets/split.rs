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
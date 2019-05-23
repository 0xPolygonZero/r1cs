use gadget_builder::GadgetBuilder;
use wire::Wire;
use witness_generator::WitnessGenerator;
use wire_values::WireValues;
use field_element::FieldElement;

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

    // TODO: Constrain bits to [0, 1].

    // TODO: Constrain weighted sum of bits.

    bit_wires
}
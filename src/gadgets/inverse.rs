use gadget_builder::GadgetBuilder;
use wire::Wire;
use witness_generator::WitnessGenerator;
use wire_values::WireValues;
use field_element::FieldElement;

fn inverse(builder: &mut GadgetBuilder, x: Wire) -> Wire {
    let inverse = builder.wire();
    builder.generator(WitnessGenerator::new(
        vec![x],
        move |values: &mut WireValues| {
            let x_value = values.get(&inverse);
            let inverse_value = x_value.multiplicative_inverse();
            values.set(inverse, inverse_value)
        },
    ));
    inverse
}
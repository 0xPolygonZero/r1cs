use gadget_builder::GadgetBuilder;
use wire::Wire;
use witness_generator::WitnessGenerator;
use wire_values::WireValues;

fn sort(builder: &mut GadgetBuilder, original: Vec<Wire>) -> Vec<Wire> {
    let n: usize = original.len();
    let sorted = builder.wires(n);

    {
        // Copying stuff into closure; see https://github.com/rust-lang/rfcs/issues/2407
        let original = original.clone();
        let sorted = sorted.clone();
        builder.generator(WitnessGenerator::new(
            original.clone(),
            move |values: &mut WireValues| {
                let mut sorted_values = values.get_all(original.iter());
                sorted_values.sort();
                values.set_all(sorted.iter(), sorted_values.iter());
            }
        ));
    }

    // TODO: Verify that `sorted` is a permutation of `original`.

    for i in 0..n-1 {
        builder.assert_le(sorted[i].into(), sorted[i + 1].into());
    }

    sorted
}
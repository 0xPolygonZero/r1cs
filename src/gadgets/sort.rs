use gadget_builder::GadgetBuilder;
use wire::Wire;
use witness_generator::WitnessGenerator;
use wire_values::WireValues;

pub fn sort(builder: &mut GadgetBuilder, original: Vec<Wire>) -> Vec<Wire> {
    let n: usize = original.len();
    let sorted = builder.wires(n);

    {
        let original = original.clone();
        let sorted = sorted.clone();

        builder.generator(WitnessGenerator::new(
            original.clone(),
            move |values: &mut WireValues| {
                let mut sorted_values = values.get_all(original.iter());
                sorted_values.sort();
                values.set_all(sorted.clone().into_iter(), sorted_values.into_iter());
            }
        ));
    }

    // TODO: Verify that `sorted` is a permutation of `original`.

    for i in 0..n-1 {
        builder.assert_le(sorted[i].into(), sorted[i + 1].into());
    }

    sorted
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;
    use wire_values::WireValues;
    use field_element::FieldElement;
    use gadgets::sort::sort;

    #[test]
    fn sort_12345() {
        let n: usize = 5;
        let mut builder = GadgetBuilder::new();
        let inputs = builder.wires(n);
        let outputs = sort(&mut builder, inputs.clone());
        let gadget = builder.build();

        let mut wire_values = WireValues::new();
        wire_values.set_all(
            inputs.clone().into_iter(),
            vec![4u128, 5, 2, 1, 3].into_iter().map(|n| n.into())
        );

        assert!(gadget.execute(&mut wire_values));
        let expected: Vec<FieldElement> = vec![1u128, 2, 3, 4, 5]
            .into_iter()
            .map(|n| n.into())
            .collect();
        assert_eq!(expected, wire_values.get_all(outputs.iter()));
    }
}
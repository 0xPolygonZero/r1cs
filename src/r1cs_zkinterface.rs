use std::collections::HashSet;
use std::io::Write;

use zkinterface::flatbuffers::{FlatBufferBuilder, WIPOffset};
use zkinterface::writing::{CircuitOwned, VariablesOwned};
use zkinterface::zkinterface_generated::zkinterface::{BilinearConstraint, BilinearConstraintArgs, Message, R1CSConstraints, R1CSConstraintsArgs, Root, RootArgs, Variables, VariablesArgs, Witness, WitnessArgs};

use crate::{Constraint, Element, Expression, Field, Gadget, Wire, WireValues};

pub fn write_circuit_and_r1cs<F: Field, W: Write>(
    gadget: &Gadget<F>,
    public_wires: &HashSet<Wire>,
    writer: &mut W,
) {
    let wires = gadget_wires(gadget);

    let circuit_owned = CircuitOwned {
        connections: VariablesOwned {
            variable_ids: public_wires.iter().map(|w| w.index as u64).collect(),
            values: None,
        },
        free_variable_id: wires.len() as u64,
        r1cs_generation: true,
        field_maximum: Some(Element::<F>::largest_element().to_biguint().to_bytes_le()),
    };

    circuit_owned.write(&mut *writer).unwrap();

    write_r1cs(gadget, writer)
}

fn write_r1cs<F: Field, W: Write>(gadget: &Gadget<F>, writer: &mut W) {
    let mut builder = FlatBufferBuilder::new();

    let constraints: Vec<_> = gadget.constraints.iter()
        .map(|c| convert_constraint(c, &mut builder))
        .collect();

    let args = R1CSConstraintsArgs {
        constraints: Some(builder.create_vector(&constraints)),
        info: None,
    };

    let r1cs_constraints = R1CSConstraints::create(&mut builder, &args);

    let root = Root::create(&mut builder, &RootArgs {
        message_type: Message::R1CSConstraints,
        message: Some(r1cs_constraints.as_union_value()),
    });

    builder.finish_size_prefixed(root, None);

    writer.write_all(builder.finished_data()).unwrap();
}

fn convert_constraint<'a, F: Field>(
    constraint: &Constraint<F>,
    builder: &mut FlatBufferBuilder<'a>
) -> WIPOffset<BilinearConstraint<'a>> {
    let Constraint { a, b, c } = constraint;
    let a_offset = convert_expression(a, builder);
    let b_offset = convert_expression(b, builder);
    let c_offset = convert_expression(c, builder);
    BilinearConstraint::create(builder, &BilinearConstraintArgs {
        linear_combination_a: Some(a_offset),
        linear_combination_b: Some(b_offset),
        linear_combination_c: Some(c_offset),
    })
}

fn convert_expression<'a, F: Field>(
    exp: &Expression<F>,
    builder: &mut FlatBufferBuilder<'a>,
) -> WIPOffset<Variables<'a>> {
    let mut variable_ids = Vec::new();
    let mut values = Vec::new();
    for (wire, coefficient) in exp.coefficients().iter() {
        variable_ids.push(wire.index as u64);
        values.extend(element_to_bytes_le(coefficient));
    }

    let ids = builder.create_vector(&variable_ids);
    let values = builder.create_vector(&values);
    Variables::create(builder, &VariablesArgs {
        variable_ids: Some(ids),
        values: Some(values),
        info: None,
    })
}

pub fn write_circuit_and_witness<F: Field, W: Write>(
    gadget: &Gadget<F>,
    witness: &WireValues<F>,
    public_wires: &HashSet<Wire>,
    writer: &mut W,
) {
    let wires = gadget_wires(gadget);

    let mut public_witness = WireValues::new();
    let mut private_witness = WireValues::new();
    for (wire, value) in witness.as_map() {
        if *wire == Wire::ONE {
            continue;
        }

        if public_wires.contains(wire) {
            public_witness.set(*wire, value.clone());
        } else {
            private_witness.set(*wire, value.clone());
        }
    }

    let circuit_owned = CircuitOwned {
        connections: convert_wire_values_to_variables_owned(&public_witness),
        free_variable_id: wires.len() as u64,
        r1cs_generation: false,
        field_maximum: None,
    };

    circuit_owned.write(&mut *writer).unwrap();

    write_private_assignment(&private_witness, writer);
}

fn write_private_assignment<F: Field, W: Write>(private_witness: &WireValues<F>, writer: &mut W) {
    let mut builder = &mut FlatBufferBuilder::new();

    let (ids, values) = convert_wire_values(private_witness);

    let ids = builder.create_vector(&ids);
    let values = builder.create_vector(&values);
    let values = Variables::create(&mut builder, &VariablesArgs {
        variable_ids: Some(ids),
        values: Some(values),
        info: None,
    });
    let assign = Witness::create(&mut builder, &WitnessArgs {
        assigned_variables: Some(values),
    });
    let message = Root::create(&mut builder, &RootArgs {
        message_type: Message::Witness,
        message: Some(assign.as_union_value()),
    });
    builder.finish_size_prefixed(message, None);

    writer.write_all(builder.finished_data()).unwrap();
}

fn convert_wire_values<F: Field>(wire_values: &WireValues<F>) -> (Vec<u64>, Vec<u8>) {
    let mut variable_ids = Vec::new();
    let mut values = Vec::new();
    for (wire, value) in wire_values.as_map().iter() {
        variable_ids.push(wire.index as u64);
        values.extend(element_to_bytes_le(value));
    }
    (variable_ids, values)
}
fn convert_wire_values_to_variables_owned<F: Field>(wire_values: &WireValues<F>) -> VariablesOwned {
    let (variable_ids, values) = convert_wire_values(wire_values);
    VariablesOwned { variable_ids, values: Some(values) }
}

fn element_to_bytes_le<F: Field>(n: &Element<F>) -> Vec<u8> {
    let mut bytes = n.to_biguint().to_bytes_le();
    while bytes.len() < bytes_per_element::<F>() {
        bytes.push(0);
    }
    bytes
}

/// Get the number of bytes required to represent any element of the given field.
fn bytes_per_element<F: Field>() -> usize {
    let bits = Element::<F>::max_bits();
    (bits + 7) / 8
}

fn gadget_wires<F: Field>(gadget: &Gadget<F>) -> HashSet<Wire> {
    let mut wires = HashSet::new();
    for constraint in &gadget.constraints {
        wires.extend(constraint.a.dependencies());
        wires.extend(constraint.b.dependencies());
        wires.extend(constraint.c.dependencies());
    }
    wires
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use num::BigUint;

    use crate::{Element, Expression, Field, GadgetBuilder, values, Wire};
    use crate::{write_circuit_and_r1cs, write_circuit_and_witness};

    #[test]
    fn serialize() {
        // Create a simple gadget for proving knowledge of an element with a given inverse.
        let mut builder = GadgetBuilder::<F7>::new();
        let x_wire = builder.wire();
        let x_exp = Expression::from(x_wire);
        let x_inverse = builder.inverse(&x_exp);
        let gadget = builder.build();

        let mut public_wires = HashSet::new();
        public_wires.insert(Wire::ONE);
        public_wires.extend(x_inverse.dependencies());

        write_circuit_and_r1cs(&gadget, &public_wires, &mut Vec::new());

        let mut wire_values = values!(x_wire => 2u8.into());
        gadget.execute(&mut wire_values);
        assert_eq!(x_inverse.evaluate(&wire_values), Element::from(4u8));

        write_circuit_and_witness(&gadget, &wire_values, &public_wires, &mut Vec::new());
    }

    #[derive(Debug)]
    struct F7 {}

    impl Field for F7 {
        fn order() -> BigUint {
            BigUint::from(7u8)
        }
    }
}

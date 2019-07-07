# Rusty Gadgets

This is a rust library for building R1CS gadgets, which are useful in SNARKs and other argument systems.

An R1CS instance is defined by three matrices, `A`, `B` and `C`. These encode the following NP-complete decision problem: does there exist a witness `w` such that `Aw ∘ Bw = Cw`?

An R1CS gadget is comprised of an R1CS instance and a witness generator which, given certain inputs, generates a complete witness which satisfies the instance.


## Types

A `Wire` represents an element of a witness vector in an R1CS instance. There are two kinds of wires: input wires and generated wires. This library does not have a notion of output wires; the output of a gadget is the entire witness vector.

An `Expression` is a linear combination of wires.

A `BooleanWire` is a `Wire` which has been constrained in such a way that it can only equal 0 or 1. Similarly, `BooleanExpression` is an `Expression` which has been constrained to be binary.

A `BinaryWire` is a vector of `BooleanWire`s. Similarly, a `BinaryExpression` is a vector of `BooleanExpression`s.


## Basic example

Here's a simple gadget which computes the cube of a field element:

```rust
// Create a gadget which takes a single input, x, and computes x*x*x.
let mut builder = GadgetBuilder::new();
let x = builder.wire();
let x_exp = Expression::from(x);
let x_squared = builder.product(&x_exp, &x_exp);
let x_cubed = builder.product(x_squared, x_exp);
let gadget = builder.build();

// This structure maps wires to their (field element) values. Since
// x is our input, we will assign it a value before executing the
// gadget. Other wires will be computed by the gadget.
let mut values = values!(x => 5.into());

// Execute the gadget and assert that all constraints were satisfied.
let constraints_satisfied = gadget.execute(&mut values);
assert!(constraints_satisfied);

// Check the result.
assert_eq!(FieldElement::from(125), x_cubed.evaluate(&values));
```


## Boolean algebra

The example above involved native field arithmetic, but this library also supports boolean algebra. For example, here is a function which implements the boolean function `Maj`, as defined in the SHA-256 specification:

```rust
fn maj(builder: &mut GadgetBuilder,
       x: BooleanExpression,
       y: BooleanExpression,
       z: BooleanExpression) -> BooleanExpression {
    let x_y = builder.and(&x, &y);
    let x_z = builder.and(&x, &z);
    let y_z = builder.and(&y, &z);
    let x_y_xor_x_z = builder.xor(x_y, x_z);
    builder.xor(x_y_xor_x_z, y_z)
}
```

## Binary operations

This library also supports bitwise operations, such as `bitwise_and`, and binary arithmetic operations, such as `binary_sum`.

TODO: Add an example of binary arithmetic.


## Non-determinism

This library also supports non-deterministic computations. For a simple example, see `GadgetBuilder`'s `inverse` method, which is defined in in `gadget_builder_arithmetic.rs`.


## Disclaimer

This code has not been thoroughly reviewed or tested, and should not be used in any production systems.

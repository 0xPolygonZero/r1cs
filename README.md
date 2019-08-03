# r1cs [![Crates.io](https://img.shields.io/crates/v/r1cs)](https://crates.io/crates/r1cs)

This is a rust library for building R1CS gadgets, which are useful in SNARKs and other argument systems.

An R1CS instance is defined by three matrices, `A`, `B` and `C`. These encode the following NP-complete decision problem: does there exist a witness vector `w` such that `Aw ∘ Bw = Cw`?

A *gadget* for some R1CS instance takes a set of inputs, which are a subset of the witness vector. If the given inputs are valid, it extends the input set into a complete witness vector which satisfies the R1CS instance.


## Types

A `Wire` represents an element of the witness vector. An `Expression` is a linear combination of wires.

A `BooleanWire` is a `Wire` which has been constrained in such a way that it can only equal 0 or 1. Similarly, a `BooleanExpression` is an `Expression` which has been so constrained.

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


## Non-determinism

Suppose we wish to compute the multiplicative inverse of a field element `x`. While this is possible to do in a deterministic arithmetic circuit, it is prohibitively expensive. What we can do instead is have the user compute `x_inv = 1 / x`, provide the result as a witness element, and add a constraint in the R1CS instance to verify that `x * x_inv = 1`.

`GadgetBuilder` supports such non-deterministic computations via its `generator` method, which can be used like so:

```rust
fn inverse(builder: &mut GadgetBuilder, x: Expression) -> Expression {
    // Create a new witness element for x_inv.
    let x_inv = builder.wire();

    // Add the constraint x * x_inv = 1.
    builder.assert_product(&x, Expression::from(x_inv),
                           Expression::one());

    // Non-deterministically generate x_inv = 1 / x.
    builder.generator(
        x.dependencies(),
        move |values: &mut WireValues| {
            let x_value = x.evaluate(values);
            let x_inv_value = x_value.multiplicative_inverse();
            values.set(x_inv, x_inv_value);
        },
    );

    // Output x_inv.
    x_inv.into()
}
```

Note that this is roughly equivalent to `GadgetBuilder`'s built-in `inverse` method, with slight modifications for readability.


## Disclaimer

This code has not been thoroughly reviewed or tested, and should not be used in any production systems.

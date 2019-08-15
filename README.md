# r1cs [![Crates.io](https://img.shields.io/crates/v/r1cs)](https://crates.io/crates/r1cs)

This is a rust library for building R1CS gadgets over prime fields, which are useful in SNARKs and other argument systems.

An R1CS instance is defined by three matrices, `A`, `B` and `C`. These encode the following NP-complete decision problem: does there exist a witness vector `w` such that `Aw ∘ Bw = Cw`?

A *gadget* for some R1CS instance takes a set of inputs, which are a subset of the witness vector. If the given inputs are valid, it extends the input set into a complete witness vector which satisfies the R1CS instance.


## Core types

`Field` is a trait representing prime fields. An `Element<F>` is an element of the prime field `F`.

A `Wire` is an element of the witness vector. An `Expression<F>` is a linear combination of wires.

A `BooleanWire` is a `Wire` which has been constrained in such a way that it can only equal 0 or 1. Similarly, a `BooleanExpression<F>` is an `Expression<F>` which has been so constrained.

A `BinaryWire` is a vector of `BooleanWire`s. Similarly, a `BinaryExpression<F>` is a vector of `BooleanExpression<F>`s.


## Basic example

Here's a simple gadget which computes the cube of a BN128 element:

```rust
// Create a gadget which takes a single input, x, and computes x*x*x.
let mut builder = GadgetBuilder::<Bn128>::new();
let x = builder.wire();
let x_exp = Expression::from(x);
let x_squared = builder.product(&x_exp, &x_exp);
let x_cubed = builder.product(x_squared, x_exp);
let gadget = builder.build();

// This structure maps wires to their (field element) values. Since
// x is our input, we will assign it a value before executing the
// gadget. Other wires will be computed by the gadget.
let mut values = values!(x => 5u8.into());

// Execute the gadget and assert that all constraints were satisfied.
let constraints_satisfied = gadget.execute(&mut values);
assert!(constraints_satisfied);

// Check the result.
assert_eq!(Element::from(125u8), x_cubed.evaluate(&values));
```


## Custom fields

You can define a custom field by implementing the `field::Field` trait. As an example, here's the definition of `Bn128` which was referenced above:

```rust
pub struct Bn128 {}

impl Field for Bn128 {
    fn order() -> BigUint {
        BigUint::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495617"
        ).unwrap()
    }
}
```


## Boolean algebra

The example above involved native field arithmetic, but this library also supports boolean algebra. For example, here is a function which implements the boolean function `Maj`, as defined in the SHA-256 specification:

```rust
fn maj<F: Field>(builder: &mut GadgetBuilder<F>,
                 x: BooleanExpression<F>,
                 y: BooleanExpression<F>,
                 z: BooleanExpression<F>) -> BooleanExpression<F> {
    let x_y = builder.and(&x, &y);
    let x_z = builder.and(&x, &z);
    let y_z = builder.and(&y, &z);
    let x_y_xor_x_z = builder.xor(x_y, x_z);
    builder.xor(x_y_xor_x_z, y_z)
}
```

## Binary operations

This library also supports bitwise operations, such as `bitwise_and`, and binary arithmetic operations, such as `binary_sum`.


## Permutation networks

To verify that two lists are permutations of one another, you can use `assert_permutation`. This is implemented using AS-Waksman permutation networks, which permute `n` items using roughly `n log_2(n) - n` switches. `assert_permutation` uses two constraints per switch: one "is boolean" check and one constraint for routing.

Permutation networks make it easy to implement sorting gadgets, which we provide in the form of `sort_ascending` and `sort_descending`.


## Non-determinism

Suppose we wish to compute the multiplicative inverse of a field element `x`. While this is possible to do in a deterministic arithmetic circuit, it is prohibitively expensive. What we can do instead is have the user compute `x_inv = 1 / x`, provide the result as a witness element, and add a constraint in the R1CS instance to verify that `x * x_inv = 1`.

`GadgetBuilder` supports such non-deterministic computations via its `generator` method, which can be used like so:

```rust
fn inverse<F: Field>(builder: &mut GadgetBuilder<F>, x: Expression<F>) -> Expression<F> {
    // Create a new witness element for x_inv.
    let x_inv = builder.wire();

    // Add the constraint x * x_inv = 1.
    builder.assert_product(&x, Expression::from(x_inv),
                           Expression::one());

    // Non-deterministically generate x_inv = 1 / x.
    builder.generator(
        x.dependencies(),
        move |values: &mut WireValues<F>| {
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

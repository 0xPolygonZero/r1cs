# Rusty Gadgets

This is a rust library for building R1CS gadgets, which are useful in SNARKs and other argument systems.

An R1CS instance is defined by three matrices, `A`, `B` and `C`. These encode the following NP-complete decision problem: does there exist a witness `w` such that `Aw ∘ Bw = Cw`?

An R1CS gadget is comprised of an R1CS instance and a witness generator which, given certain inputs, generates a complete witness which satisfies the instance.


## Basic example

Here's a simple gadget which computes the cube of a field element:

```rust
// Create a gadget which takes a single input, x, and computes x*x*x.
let mut builder = GadgetBuilder::new();
let x = builder.wire();
let x_squared = builder.product(x.into(), x.into());
let x_cubed = builder.product(x_squared, x.into());
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


## Binary arithmetic

The example above involved native field arithmetic, but this library also supports binary arithmetic.

TODO: Add an example.


## Non-determinism

This library also supports non-deterministic computations. For a simple example, see `GadgetBuilder`'s `inverse` method, which is defined in in `gadget_builder_arithmetic.rs`.


## Disclaimer

This code has not been thoroughly reviewed or tested, and should not be used in any production systems.

#[cfg(feature = "no-std")]
use alloc::vec::Vec;
#[cfg(feature = "no-std")]
use alloc::boxed::Box;

use crate::field::Field;
use crate::wire::Wire;
use crate::wire_values::WireValues;

/// Generates some elements of the witness.
pub struct WitnessGenerator<F: Field> {
    inputs: Vec<Wire>,
    generator: Box<dyn Fn(&mut WireValues<F>)>,
}

impl<F: Field> WitnessGenerator<F> {
    /// Creates a new `WitnessGenerator`.
    ///
    /// # Arguments
    /// * `inputs` - the wires whose values must be set before this generator can run
    /// * `generate` - a function which generates some elements of the witness
    pub fn new<T>(inputs: Vec<Wire>, generate: T) -> Self
        where T: Fn(&mut WireValues<F>) + 'static {
        WitnessGenerator {
            inputs,
            generator: Box::new(generate),
        }
    }

    /// The wires whose values must be set before this generator can run.
    pub fn inputs(&self) -> &[Wire] {
        &self.inputs
    }

    /// Run the generator.
    pub fn generate(&self, values: &mut WireValues<F>) {
        (*self.generator)(values)
    }
}
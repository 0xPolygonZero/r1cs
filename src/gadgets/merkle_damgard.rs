use std::str::FromStr;

use field_element::FieldElement;
use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;

type CompressionFunction = fn(&mut GadgetBuilder, LinearCombination, LinearCombination)
                              -> LinearCombination;

impl GadgetBuilder {
    /// Creates a Merkle–Damgård hash function from the given one-way compression function.
    pub fn merkle_damgard<'a, T>(&mut self, blocks: T, compress: CompressionFunction)
                                 -> LinearCombination
        where T: IntoIterator<Item=&'a LinearCombination> {
        let mut current = LinearCombination::from(initial_value());
        let mut len = 0;
        for block in blocks {
            current = compress(self, current, block.clone());
            len += 1;
        }

        // Length padding
        compress(self, current, LinearCombination::from(len))
    }
}

fn initial_value() -> FieldElement {
    // This is SHA-256("Daniel") % FieldElement::size()
    FieldElement::from_str("8055487882410849247272479787595810817356819330244461513770925873713345838208").unwrap()
}
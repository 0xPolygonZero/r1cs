use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;
use wire_values::WireValues;
use gadgets::merkle_trees::{TrieDeletionProof, TrieInsertionProof};

/// The depth of the record commitment trie.
const RECORD_DEPTH: usize = 64;

/// The number of commitments that can be added to, and removed from, the record commitment trie in
/// each block.
const RECORD_UPDATES: usize = 100;

/// The maximum size of the validator pool.
const NUM_VALIDATORS: usize = 100;

struct Inputs {
    /// The Merkle root of the record commitment set.
    records_root: LinearCombination,
    /// A list of record hashes corresponding to validator accounts.
    validator_list: [LinearCombination; NUM_VALIDATORS],
    record_commitments_added: [TrieInsertionProof; RECORD_UPDATES],
    record_commitments_removed: [TrieDeletionProof; RECORD_UPDATES],
}

struct Outputs {
    records_root: LinearCombination,
    validators_root: LinearCombination,
}

/// Validates a block, and returns the updated blockchain state.
fn mir_block(builder: &mut GadgetBuilder, inputs: Inputs) -> Outputs {
    builder.generator(
        vec![], // TODO: deps
        move |values: &mut WireValues| {
            ;
        },
    );

    unimplemented!("TODO")
}

#[cfg(test)]
mod tests {
    #[test]
    fn mir_block() {
        ;
    }
}
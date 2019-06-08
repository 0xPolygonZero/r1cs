use gadget_builder::GadgetBuilder;
use gadgets::merkle_proofs::MerklePath;
use linear_combination::LinearCombination;
use wire_values::WireValues;

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
    old_records_root: LinearCombination,
    old_active_validators_root: LinearCombination,
    old_next_validators_root: LinearCombination,
    record_commitments_removed: [MerklePath; RECORD_UPDATES],
    record_commitments_added: [MerklePath; RECORD_UPDATES],
}

struct Outputs {
    new_records_root: LinearCombination,
    new_active_validators_root: LinearCombination,
    new_next_validators_root: LinearCombination,
}

/// Validates a block, and returns the updated blockchain state.
fn mir_block(builder: &mut GadgetBuilder, inputs: Inputs) -> Outputs {
    builder.generator(
        vec![], // TODO: deps
        move |values: &mut WireValues| {
            // TODO
        },
    );

    let mut records_root = inputs.old_records_root;
    let compress = GadgetBuilder::mimc_compress;
    for inserted_path in inputs.record_commitments_added.iter() {
        let (before, after) = builder.merkle_trie_insert(inserted_path.clone(), compress);
        builder.assert_equal(records_root, before);
        records_root = after;
    }
    for deleted_path in inputs.record_commitments_removed.iter() {
        let (before, after) = builder.merkle_trie_delete(deleted_path.clone(), compress);
        builder.assert_equal(records_root, before);
        records_root = after;
    }

    Outputs {
        new_records_root: records_root,
        new_active_validators_root: LinearCombination::zero(), // TODO
        new_next_validators_root: LinearCombination::zero(), // TODO
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mir_block() {
        // TODO
    }
}
use linear_combination::LinearCombination;
use mir::record::{RecordOpening, record_commitment};
use gadget_builder::GadgetBuilder;

struct Inputs {
    births: Vec<Birth>,
    deaths: Vec<Death>,
    mutations: Vec<Mutation>,
}

struct Birth {
    record_opening: RecordOpening,
}

struct Death {
    record_opening: RecordOpening,
}

struct Mutation {
    old_record_opening: RecordOpening,
    new_record_opening: RecordOpening,
}

struct Outputs {
    record_commitments_added: Vec<LinearCombination>,
    record_commitments_removed: Vec<LinearCombination>,
}

fn mir_transaction(builder: &mut GadgetBuilder, inputs: Inputs) -> Outputs {
    let mut record_commitments_added = Vec::new();
    let mut record_commitments_removed = Vec::new();

    // TODO: Verify the predicates of all records involved in this transaction.

    for birth in inputs.births {
        record_commitments_added.push(record_commitment(builder, birth.record_opening));
    }

    for death in inputs.deaths {
        record_commitments_removed.push(record_commitment(builder, death.record_opening));
    }

    for mutation in inputs.mutations {
        record_commitments_removed.push(record_commitment(builder, mutation.old_record_opening));
        record_commitments_added.push(record_commitment(builder, mutation.new_record_opening));
    }

    Outputs { record_commitments_added, record_commitments_removed }
}
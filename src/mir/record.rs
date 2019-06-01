use linear_combination::LinearCombination;
use gadget_builder::GadgetBuilder;

pub struct Record {
    pub birth_predicate: LinearCombination,
    pub death_predicate: LinearCombination,
    pub mutation_predicate: LinearCombination,
    pub log_predicate: LinearCombination,
    pub payload: LinearCombination,
}

impl Record {
    /// Convert this record into a vector of field element blocks.
    pub fn serialize(&self) -> Vec<LinearCombination> {
        vec![
            self.birth_predicate.clone(),
            self.death_predicate.clone(),
            self.mutation_predicate.clone(),
            self.log_predicate.clone(),
            self.payload.clone(),
        ]
    }
}

pub struct RecordOpening {
    pub record: Record,
    pub opening: LinearCombination,
}

pub fn record_commitment(builder: &mut GadgetBuilder, record_opening: RecordOpening)
                     -> LinearCombination {
    let hash = record_hash(builder, record_opening.record);
    builder.mimc_compress(hash, record_opening.opening)
}

pub fn record_hash(builder: &mut GadgetBuilder, record: Record) -> LinearCombination {
    builder.mimc_hash(&record.serialize())
}
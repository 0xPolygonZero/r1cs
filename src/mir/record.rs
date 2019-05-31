use linear_combination::LinearCombination;

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
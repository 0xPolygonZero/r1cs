use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;

impl GadgetBuilder {
    /// Verify a membership proof for any binary Merkle tree.
    pub fn verify_merkle_membership_proof<'a, T>(proof: T)
        where T: IntoIterator<Item=&'a MerkleMembershipProofPart> {
        unimplemented!("TODO")
    }
}

struct MerkleMembershipProofPart {
    is_left: LinearCombination,
    other_hash: LinearCombination,
}
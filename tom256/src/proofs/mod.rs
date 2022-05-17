mod equality;
mod exp;
mod membership;
mod multiplication;
mod point_add;
mod utils;

pub use exp::{ExpProof, PointExpSecrets};
pub use membership::MembershipProof;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
pub struct ProofInput {
    pub msg_hash: String,
    pub pubkey: String,
    pub signature: String,
    pub index: usize,
    pub ring: Vec<String>,
}

/*
pub struct ParsedProofInput<C, CC> {
    pub msg_hash: Scalar<C>,
    pub pubkey: Point<C>,
    pub signature: Signature<C>,
    pub index: usize,
    pub ring: Vec<Scalar<CC>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ZkAttestProof<C, CC> {
    pub pedersen: PedersenCycle<C, CC>,
    pub msg_hash: Scalar<C>,
    pub r_sig: Point<C>,
    pub commitment_to_s_sig: Point<C>,
    pub commitment_to_address: Point<CC>,
    pub commitment_to_pk_x: Point<CC>,
    pub commitment_to_pk_y: Point<CC>,
    pub signature_proof: ExpProof<C, CC>,
    pub membership_proof: MembershipProof<CC>,
    pub ring: Vec<Scalar<CC>>,
}
*/

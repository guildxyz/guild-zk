mod equality;
mod exp;
mod membership;
mod multiplication;
mod point_add;
mod utils;

pub use exp::{ExpProof, PointExpSecrets};
pub use membership::MembershipProof;

/*
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

use bls::{G2Affine, Scalar};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Address([u8; 32]);

impl From<G2Affine> for Address {
    fn from(pubkey: G2Affine) -> Self {
        Self::from(&pubkey)
    }
}

impl From<&G2Affine> for Address {
    fn from(pubkey: &G2Affine) -> Self {
        let address_bytes = crate::hash::hash_to_fp(&pubkey.to_compressed()).to_bytes();
        Self(address_bytes)
    }
}

impl Address {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn as_scalar(&self) -> Scalar {
        // NOTE unwrap is fine because a valid address
        // can only be created from a Scalar type via
        // hash_to_fp
        Scalar::from_bytes(&self.0).unwrap()
    }
}

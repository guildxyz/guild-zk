use bls::{G1Affine, G1Projective, G2Affine};

#[derive(Debug, Clone, Copy)]
pub struct Signature(G1Affine);

impl Signature {
    pub fn new(sig: G1Affine) -> Self {
        Self(sig)
    }

    pub fn inner(&self) -> &G1Affine {
        &self.0
    }

    pub fn verify(&self, msg: &[u8], vk: &G2Affine) -> bool {
        let msg_hash_g1 = crate::hash::hash_to_g1(msg);
        bls::pairing(&msg_hash_g1, vk) == bls::pairing(&self.0, &G2Affine::generator())
    }
}

impl From<G1Affine> for Signature {
    fn from(sig: G1Affine) -> Self {
        Self(sig)
    }
}

impl From<G1Projective> for Signature {
    fn from(sig: G1Projective) -> Self {
        Self(sig.into())
    }
}

use bls::{G1Affine, G2Affine, Scalar};
use ff::Field;
use rand_core::{CryptoRng, RngCore};

pub struct Keypair {
    privkey: Scalar,
    pubkey: G2Affine,
}

impl Keypair {
    pub fn new(privkey: Scalar) -> Self {
        Self {
            privkey,
            pubkey: G2Affine::from(G2Affine::generator() * privkey),
        }
    }

    pub fn new_checked(privkey: Scalar, pubkey: G2Affine) -> Result<Self, String> {
        if pubkey == (G2Affine::generator() * privkey).into() {
            Ok(Self { privkey, pubkey })
        } else {
            Err("invalid keypair".to_string())
        }
    }

    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let privkey = Scalar::random(rng);
        Self::new(privkey)
    }

    pub fn pubkey(&self) -> G2Affine {
        self.pubkey
    }

    pub fn privkey(&self) -> &Scalar {
        &self.privkey
    }

    pub fn sign(&self, msg: &[u8]) -> G1Affine {
        let msg_hash_g1 = crate::hash::hash_to_g1(msg);
        (msg_hash_g1 * self.privkey).into()
    }
}

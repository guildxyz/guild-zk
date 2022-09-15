use crate::signature::Signature;
use bls::{G2Affine, Scalar};
use ff::Field;
use rand_core::{CryptoRng, RngCore};
use thiserror::Error;

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum KeypairError {
    #[error("pubkey-privkey mismatch")]
    InvalidKeypair,
}

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

    pub fn new_checked(privkey: Scalar, pubkey: G2Affine) -> Result<Self, KeypairError> {
        if pubkey != (G2Affine::generator() * privkey).into() {
            Err(KeypairError::InvalidKeypair)
        } else {
            Ok(Self { privkey, pubkey })
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

    pub fn sign(&self, msg: &[u8]) -> Signature {
        let msg_hash_g1 = crate::hash::hash_to_g1(msg);
        Signature::new((msg_hash_g1 * self.privkey).into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand_core::OsRng;

    #[test]
    fn signature() {
        let keypair = Keypair::random(&mut OsRng);
        let msg = b"message to be signed";
        let signature = keypair.sign(msg);
        assert!(signature.verify(msg, &keypair.pubkey));
        // wrong message
        assert!(!signature.verify(&[23; 32], &keypair.pubkey));
        let other_keypair = Keypair::random(&mut OsRng);
        // wrong verifying key
        assert!(!signature.verify(msg, &other_keypair.pubkey));
    }
}

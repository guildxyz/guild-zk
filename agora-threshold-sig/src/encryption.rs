use crate::hash::hash_to_fp;
use bls::{G2Affine, Scalar};
use chacha::{
    aead::{Aead, AeadCore},
    ChaCha20Poly1305 as ChaCha, KeyInit, Nonce,
};
use ff::Field;
use rand_core::{CryptoRng, RngCore};

#[derive(Debug, Clone)]
pub struct Encryption {
    ciphertext: Vec<u8>,
    ephemeral_pubkey: G2Affine,
    nonce: Nonce,
}

impl Encryption {
    pub fn new<R: CryptoRng + RngCore>(
        mut rng: R,
        msg: &[u8],
        pubkey: G2Affine,
    ) -> Result<Self, String> {
        let ephemeral_privkey = Scalar::random(&mut rng);
        let ephemeral_pubkey = G2Affine::from(G2Affine::generator() * ephemeral_privkey);
        let encryption_pubkey = G2Affine::from(pubkey * ephemeral_privkey);
        let encryption_key = hash_to_fp(&encryption_pubkey.to_compressed()).to_bytes();

        let cipher = ChaCha::new(&encryption_key.into());
        let nonce = ChaCha::generate_nonce(&mut rng);
        let ciphertext = cipher.encrypt(&nonce, msg).map_err(|e| e.to_string())?;

        Ok(Self {
            ciphertext,
            ephemeral_pubkey,
            nonce,
        })
    }

    pub fn decrypt(&self, privkey: &Scalar) -> Result<Vec<u8>, String> {
        let encryption_pubkey = G2Affine::from(self.ephemeral_pubkey * privkey);
        let encryption_key = hash_to_fp(&encryption_pubkey.to_compressed()).to_bytes();

        let cipher = ChaCha::new(&encryption_key.into());
        cipher
            .decrypt(&self.nonce, self.ciphertext.as_ref())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::keypair::Keypair;
    use rand_core::OsRng;

    #[test]
    fn encryption_and_decryption() {
        let msg = b"hello_world!!!!";
        let other_msg = b"to whom it may concern";
        let mut rng = OsRng;
        let keypair = Keypair::random(&mut rng);
        let other_keypair = Keypair::random(&mut rng);

        let encryption = Encryption::new(rng, msg, keypair.pubkey()).unwrap();
        let decrypted = encryption.decrypt(keypair.privkey()).unwrap();
        assert_eq!(decrypted, msg);
        assert!(encryption.decrypt(other_keypair.privkey()).is_err());
        let encryption = Encryption::new(rng, other_msg, other_keypair.pubkey()).unwrap();
        let decrypted = encryption.decrypt(other_keypair.privkey()).unwrap();
        assert_eq!(decrypted, other_msg);
        assert!(encryption.decrypt(keypair.privkey()).is_err());
    }
}

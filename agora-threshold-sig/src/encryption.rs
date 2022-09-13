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
    pub ciphertext: Vec<u8>,
    pub ephemeral_pubkey: G2Affine,
    pub nonce: Nonce,
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

    pub fn decrypt(&self, encryption_key: &[u8]) -> Result<Vec<u8>, String> {
        let cipher = ChaCha::new(encryption_key.into());
        cipher
            .decrypt(&self.nonce, self.ciphertext.as_ref())
            .map_err(|e| e.to_string())
    }

    pub fn decrypt_with_pubkey(&self, pubkey: &G2Affine) -> Result<Vec<u8>, String> {
        let encryption_key = hash_to_fp(&pubkey.to_compressed()).to_bytes();
        self.decrypt(&encryption_key)
    }

    pub fn decrypt_with_privkey(&self, privkey: &Scalar) -> Result<Vec<u8>, String> {
        let encryption_pubkey = G2Affine::from(self.ephemeral_pubkey * privkey);
        self.decrypt_with_pubkey(&encryption_pubkey)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::keypair::Keypair;
    use agora_interpolate::Polynomial;
    use bls::G2Projective;
    use rand_core::OsRng;

    #[test]
    fn encryption_and_decryption() {
        let msg = b"hello_world!!!!";
        let other_msg = b"to whom it may concern";
        let mut rng = OsRng;
        let keypair = Keypair::random(&mut rng);
        let other_keypair = Keypair::random(&mut rng);

        let encryption = Encryption::new(rng, msg, keypair.pubkey()).unwrap();
        let decrypted = encryption.decrypt_with_privkey(keypair.privkey()).unwrap();
        assert_eq!(decrypted, msg);
        assert!(encryption
            .decrypt_with_privkey(other_keypair.privkey())
            .is_err());
        let encryption = Encryption::new(rng, other_msg, other_keypair.pubkey()).unwrap();
        let decrypted = encryption
            .decrypt_with_privkey(other_keypair.privkey())
            .unwrap();
        assert_eq!(decrypted, other_msg);
        assert!(encryption.decrypt_with_privkey(keypair.privkey()).is_err());
    }

    #[test]
    fn non_deterministic_encryption() {
        let msg = b"peepo";
        let pubkey = G2Affine::generator();
        let encryption = Encryption::new(OsRng, msg, pubkey).unwrap();
        for _ in 0..10 {
            let other_encryption = Encryption::new(OsRng, msg, pubkey).unwrap();
            assert_ne!(&encryption.ciphertext, &other_encryption.ciphertext);
        }
    }

    #[test]
    fn decryption_from_shares() {
        let n = 4;
        let id_vec = (0..n)
            .map(|i| Scalar::from((10 + i) as u64))
            .collect::<Vec<Scalar>>();
        // generate coefficients for polynomials
        let private_coeffs = id_vec
            .iter()
            .map(|_| Scalar::random(OsRng))
            .collect::<Vec<Scalar>>();
        let public_coeffs = private_coeffs
            .iter()
            .map(|s| G2Affine::generator() * s)
            .collect::<Vec<G2Projective>>();

        let private_poly = Polynomial::new(private_coeffs);
        let public_poly = Polynomial::new(public_coeffs);

        // generate test keypairs via evaluating the polynomials
        let share_keypairs = id_vec
            .iter()
            .map(|id| {
                let privkey = private_poly.evaluate(id);
                let pubkey = public_poly.evaluate(id);
                Keypair::new_checked(privkey, pubkey.into()).unwrap()
            })
            .collect::<Vec<Keypair>>();

        // encrypt plaintext with the public verification key (0th poly coeff)
        let msg = b"this is the plaintext";
        let global_public_key = public_poly.coeffs()[0];
        let encryption = Encryption::new(OsRng, msg, global_public_key.into()).unwrap();
        // collect decryption key shares
        let decryption_shares = share_keypairs
            .iter()
            .map(|keypair| encryption.ephemeral_pubkey * keypair.privkey())
            .collect::<Vec<G2Projective>>();
        // interpolate to get the decryption key
        let decryption_pubkey = Polynomial::interpolate(&id_vec, &decryption_shares)
            .unwrap()
            .coeffs()[0];

        let decrypted = encryption
            .decrypt_with_pubkey(&decryption_pubkey.into())
            .unwrap();
        assert_eq!(decrypted, msg);
        // not enough shares collected
        let decryption_pubkey =
            Polynomial::interpolate(&id_vec[0..n - 1], &decryption_shares[0..n - 1])
                .unwrap()
                .coeffs()[0];

        assert!(encryption
            .decrypt_with_pubkey(&decryption_pubkey.into())
            .is_err());
    }
}

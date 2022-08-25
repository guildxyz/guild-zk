use crate::hash::*;
use bls::{pairing, G1Affine, G2Affine, Scalar};
use ff::Field;
use rand_core::{CryptoRng, RngCore};
use zeroize::Zeroize;

#[derive(Clone, Copy, Debug)]
pub struct EncryptedShare {
    pub c: Scalar,
    pub U: G2Affine,
    pub V: G1Affine,
}

impl EncryptedShare {
    #[allow(unused_assignments)]
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        id: &[u8],
        pubkey: &G2Affine,
        secret_share: &Scalar,
    ) -> Self {
        let mut r = Scalar::random(rng);
        let mut Q = hash_to_g1(id); // instead of hashing the whole participant?

        let mut e = pairing(&Q, &G2Affine::from(pubkey * r));
        let mut eh = hash_to_fp(e.to_string().as_bytes());

        let c = secret_share + eh;
        let U = G2Affine::from(G2Affine::generator() * r);
        let mut H = hash_to_g1(
            format!(
                "{:?}.{:?}.{:?}",
                Q.to_compressed(),
                c.to_bytes(),
                U.to_compressed()
            )
            .as_bytes(),
        );

        let V = G1Affine::from(H * (eh * r.invert().unwrap()));

        // zeroize before dropping
        r.zeroize();
        eh.zeroize();
        Q.zeroize();
        H.zeroize();
        // NOTE Gt doesn't have Zeroize implemented, so just assign
        // identity to this. Is this the right way though -> unused assignment?
        e = Default::default();

        Self { c, U, V }
    }

    pub fn verify(&self, id: &[u8], public_share: &G2Affine) -> bool {
        let Q = hash_to_g1(id);
        let H = hash_to_g1(
            format!(
                "{:?}.{:?}.{:?}",
                Q.to_compressed(),
                self.c.to_bytes(),
                self.U.to_compressed()
            )
            .as_bytes(),
        );

        let g2c = G2Affine::from(G2Affine::generator() * self.c);
        let e1 = pairing(&H, &g2c);

        let share_pairing = pairing(&H, public_share);
        let verification_pairing = pairing(&self.V, &self.U);
        // NOTE in the bls crate, multiplication is implemented as addition
        // NOTE but under the hood gt1 + gt2 looks like gt1.0 * gt2.0
        let e2 = share_pairing + verification_pairing;

        e1 == e2
    }

    #[allow(unused_assignments)]
    pub fn decrypt(&self, id: &[u8], secret_key: &Scalar) -> Scalar {
        let mut Q = hash_to_g1(id);
        let mut e = pairing(&G1Affine::from(Q * secret_key), &self.U);
        let mut eh = hash_to_fp(e.to_string().as_bytes());

        let result = self.c - eh;
        // NOTE Gt doesn't have Zeroize implemented, so just assign
        // identity to this. Is this the right way though -> unused assignment?
        e = Default::default();
        eh.zeroize();
        Q.zeroize();
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Share {
        public: G2Affine,
        secret: Scalar,
    }

    impl Share {
        fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
            let secret = Scalar::random(rng);
            Self {
                public: G2Affine::from(G2Affine::generator() * secret),
                secret,
            }
        }
    }

    #[test]
    fn verify_and_decrypt() {
        let mut rng = rand_core::OsRng;
        let g2 = G2Affine::generator();
        let secret_key = Scalar::random(&mut rng);
        let id_bytes = Scalar::random(&mut rng).to_bytes();
        let pubkey = G2Affine::from(g2 * secret_key);

        let share = Share::random(&mut rng);

        let encrypted_share = EncryptedShare::new(&mut rng, &id_bytes, &pubkey, &share.secret);
        assert!(encrypted_share.verify(&id_bytes, &share.public));
        let decrypted_share = encrypted_share.decrypt(&id_bytes, &secret_key);

        assert_eq!(share.secret, decrypted_share);

        let invalid_share = encrypted_share.decrypt(&id_bytes, &Scalar::random(&mut rng));
        assert_ne!(share.secret, invalid_share);

        let invalid_secret_share = Scalar::random(&mut rng);
        let invalid_public_share = G2Affine::from(g2 * invalid_secret_share);
        assert!(!encrypted_share.verify(&id_bytes, &invalid_public_share))
    }
}

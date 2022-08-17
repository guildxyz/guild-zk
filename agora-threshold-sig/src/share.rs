use crate::*;
use bls::{pairing, G1Affine, G2Affine, Scalar};
use ff::Field;
use rand_core::RngCore;
use zeroize::Zeroize;

#[derive(Clone, Debug)]
pub struct Participant {
    id: Scalar,
    pubkey: G2Affine,
}

impl Participant {
    fn to_bytes(&self) -> [u8; FP_BYTES + G2_BYTES] {
        let mut bytes = [0u8; FP_BYTES + G2_BYTES];
        bytes[0..FP_BYTES].copy_from_slice(&self.id.to_bytes());
        bytes[FP_BYTES..].copy_from_slice(&self.pubkey.to_compressed());
        bytes
    }
}

pub struct EncryptedShare {
    pub c: Scalar,
    pub U: G2Affine,
    pub V: G1Affine,
}

impl EncryptedShare {
    #[allow(unused_assignments)]
    pub fn new<R: RngCore>(rng: &mut R, participant: &Participant, secret_share: &Scalar) -> Self {
        let mut r = Scalar::random(rng);
        let mut Q = hash_to_g1(&participant.to_bytes());

        let mut e = pairing(&Q, &G2Affine::from(participant.pubkey * r));
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
        // identity to this. Is this the right way though?
        e = Default::default();

        Self { c, U, V }
    }

    pub fn verify(&self, participant: &Participant, public_share: &G2Affine) -> bool {
        let Q = hash_to_g1(&participant.to_bytes());
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

    pub fn decrypt(&self, participant: &Participant, secret_key: &Scalar) -> Scalar {
        let Q = hash_to_g1(&participant.to_bytes());
        let e = pairing(&G1Affine::from(Q * secret_key), &self.U);
        let eh = hash_to_fp(e.to_string().as_bytes());

        self.c - eh
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand_core::SeedableRng;
    use rand_xorshift::XorShiftRng;

    struct Share {
        public: G2Affine,
        secret: Scalar,
    }

    impl Share {
        fn random<R: RngCore>(rng: &mut R) -> Self {
            let secret = Scalar::random(rng);
            Self {
                public: G2Affine::from(G2Affine::generator() * secret),
                secret,
            }
        }
    }

    const SEED: [u8; 16] = [0; 16];

    #[test]
    fn verify_and_decrypt() {
        let mut rng = XorShiftRng::from_seed(SEED);

        let g2 = G2Affine::generator();
        let secret_key = Scalar::random(&mut rng);
        let participant = Participant {
            id: Scalar::random(&mut rng),
            pubkey: G2Affine::from(g2 * secret_key),
        };

        let share = Share::random(&mut rng);

        let encrypted_share = EncryptedShare::new(&mut rng, &participant, &share.secret);
        assert!(encrypted_share.verify(&participant, &share.public));
        let decrypted_share = encrypted_share.decrypt(&participant, &secret_key);

        assert_eq!(share.secret, decrypted_share);

        let invalid_share = encrypted_share.decrypt(&participant, &Scalar::random(&mut rng));
        assert_ne!(share.secret, invalid_share);

        let invalid_secret_share = Scalar::random(&mut rng);
        let invalid_public_share = G2Affine::from(g2 * invalid_secret_share);
        assert!(!encrypted_share.verify(&participant, &invalid_public_share))
    }
}

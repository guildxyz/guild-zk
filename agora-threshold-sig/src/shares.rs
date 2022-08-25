use crate::encrypt::EncryptedShare;
use crate::participant::Participant;

use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct Shares {
    pub poly: Polynomial<G2Projective>,
    /// These are the encrypted shares of the privately generated shares $sh_i$
    /// with $i\in[0..n - 1]$, where $n$ is the total number of participants in
    /// the threshold cryptosystem.
    pub esh_vec: Vec<EncryptedShare>,
}

pub type ShareVerificationKeys = Vec<G2Projective>;

impl Shares {
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        private_poly: &Polynomial<Scalar>,
        participants: &[Participant],
    ) -> Self {
        let esh_vec = participants
            .iter()
            .map(|participant| {
                let secret_share = private_poly.evaluate(participant.id);
                // encrypt with other participant's pubkey so they can decrypt for themselves
                EncryptedShare::new(rng, participant, &secret_share)
            })
            .collect();
        let poly = Polynomial::new(
            private_poly
                .coeffs()
                .iter()
                .map(|coeff| G2Affine::generator() * coeff)
                .collect(),
        );

        Self { poly, esh_vec }
    }

    pub fn verification_keys(&self, participants: &[Participant]) -> ShareVerificationKeys {
        participants
            .iter()
            .map(|participant| self.poly.evaluate(participant.id))
            .collect::<ShareVerificationKeys>()
    }
}

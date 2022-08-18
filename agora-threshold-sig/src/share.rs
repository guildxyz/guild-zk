use crate::encrypt::EncryptedShare;
use crate::participant::Participant;

use agora_interpolate::Polynomial;
use bls::{G2Affine, Scalar};
use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct Shares {
    coeff_pubkeys: Vec<G2Affine>,
    encrypted_shares: Vec<EncryptedShare>,
}

impl Shares {
    pub fn generate<R: RngCore + CryptoRng>(
        rng: &mut R,
        poly: &Polynomial<Scalar>,
        participants: &[Participant],
    ) -> Self {
        // TODO size checks?
        // t = poly.coeffs().len()
        // n = participants.len()
        let encrypted_shares = participants
            .iter()
            .map(|participant| {
                let secret_share = poly.evaluate(participant.id);
                EncryptedShare::new(rng, participant, &secret_share)
            })
            .collect();
        let coeff_pubkeys = poly
            .coeffs()
            .iter()
            .map(|coeff| G2Affine::from(G2Affine::generator() * coeff))
            .collect();

        Self {
            coeff_pubkeys,
            encrypted_shares,
        }
    }
}

use crate::encrypt::EncryptedShare;
use crate::participant::Participant;

use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct Shares {
    pub coeff_pubkeys: Polynomial<G2Projective>,
    pub encrypted_shares: Vec<EncryptedShare>,
}

pub type PubkeyShares = Vec<G2Projective>;

impl Shares {
    pub fn generate_encrypted<R: RngCore + CryptoRng>(
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
        let coeff_pubkeys = Polynomial::new(
            poly.coeffs()
                .iter()
                .map(|coeff| G2Affine::generator() * coeff)
                .collect(),
        );

        Self {
            coeff_pubkeys,
            encrypted_shares,
        }
    }

    pub fn pubkey_shares(&self, participants: &[Participant]) -> PubkeyShares {
        participants
            .iter()
            .map(|participant| self.coeff_pubkeys.evaluate(participant.id))
            .collect::<Vec<G2Projective>>()
    }
}

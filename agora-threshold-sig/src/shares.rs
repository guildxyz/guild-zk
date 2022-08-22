use crate::encrypt::EncryptedShare;
use crate::participant::Participant;

use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct Shares {
    pub public_poly: Polynomial<G2Projective>,
    pub encrypted_shares: Vec<EncryptedShare>,
}

pub type Evaluations = Vec<G2Projective>;

impl Shares {
    pub fn new<R: RngCore + CryptoRng>(
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
        let public_poly = Polynomial::new(
            poly.coeffs()
                .iter()
                .map(|coeff| G2Affine::generator() * coeff)
                .collect(),
        );

        Self {
            public_poly,
            encrypted_shares,
        }
    }

    pub fn evaluations(&self, participants: &[Participant]) -> Evaluations {
        participants
            .iter()
            .map(|participant| self.public_poly.evaluate(participant.id))
            .collect::<Vec<G2Projective>>()
    }
}

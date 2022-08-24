use crate::encrypt::EncryptedShare;
use crate::participant::Participant;

use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct Shares {
    /// This is the public version $A(x)$ of the privately generated
    /// polynomial $a(x)$ with degree $t-1$, where $t$ is the threshold.
    ///
    /// The private polynomial is generated over a finite field $\mathbb{F}_p$
    ///
    /// $$a(x) = a_0 + a_1x +\ldots + a_{t - 1}x^{t - 1}$$
    ///
    /// with $x, a_i\in\mathbb{F_p}\ \forall i$. The public polynomial is defined as
    ///
    /// $$A(x) = A_0 + A_1x +\ldots + A_{t - 1}x^{t - 1}$$
    ///
    /// with $x\in\mathbb{F_p}$ and $A_i = g_2^{a_i}\in\mathbb{G_2}\ \forall i$.
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
        // TODO size checks?
        // t = poly.coeffs().len()
        // n = participants.len()
        let esh_vec = participants
            .iter()
            .map(|participant| {
                let secret_share = private_poly.evaluate(participant.id);
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

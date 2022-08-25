use crate::participant::Participant;
use agora_interpolate::Polynomial;
use bls::{G2Affine, Scalar};
use ff::Field;
use rand_core::{CryptoRng, RngCore};

use std::collections::BTreeMap;

// assume sorted?
pub struct Share {
    vk_vec: Vec<G2Affine>,
    esh_vec: Vec<EncryptedShare>,
}

/*
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
*/
pub struct Node {
    private_key: Scalar,
    private_poly: Polynomial<Scalar>,
    participant: Participant,
    shares: BTreeMap<Scalar, Share>,
}

impl Node {
    // Copy is required due to Scalar::random(r: impl RngCore) which will
    // reborrow &mut R as &mut *rng, meaning that rng is dereferenced and
    // thus moved if it's not Copy
    pub fn new<R: RngCore + CryptoRng + Copy>(rng: R, n: usize, t: usize) -> Self {
        assert!(
            n >= t,
            "threshold is greater than the total number of participants"
        );
        let private_key = Scalar::random(rng);
        let participant = Participant {
            id: Scalar::random(rng),
            pubkey: G2Affine::from(G2Affine::generator() * private_key),
        };
        let private_poly = Polynomial::new(
            (0..t)
                .into_iter()
                .map(|_| Scalar::random(rng))
                .collect::<Vec<Scalar>>(),
        );

        let mut shares = BTreeMap::with_capacity(n);
        // TODO insert own shares
        shares.insert();

        Self {
            private_key,
            private_poly,
            participant,
            peer_public_shares,
        }
    }

    pub fn generate_shares<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        participants: &[Participant],
    ) {
        let shares = Shares::new(rng, &self.private_poly, participants);
        Some(shares);
    }

    pub fn collect_share(&mut self, participant: Participant, shares: Shares) {
        let id_bytes = participant.id.to_bytes();
        if self.peer_public_shares.get(&id_bytes).is_none() {
            self.peer_public_shares.insert(
                id_bytes,
                Peer {
                    participant,
                    shares,
                },
            );
        }
    }

    // TODO verify shares

    /// Attempts to recover our own share signing key.
    pub fn recover_keys(&self) -> (Scalar, G2Affine) {}
}
// methods?
//shsk: Scalar, // share signing key
//shvk: G2Affine, // share verification key

#[test]
fn dkg_23() {
    let mut rng = rand_core::OsRng;
    let secret_keys = (0..3)
        .into_iter()
        .map(|_| Scalar::random(&mut rng))
        .collect::<Vec<Scalar>>();

    let participants = secret_keys
        .iter()
        .enumerate()
        .map(|(i, private_key)| Participant {
            id: Scalar::from(i as u64),
            pubkey: G2Affine::from(G2Affine::generator() * private_key),
        })
        .collect::<Vec<Participant>>();

    let private_polys = participants
        .iter()
        .map(|_| Polynomial::new(vec![Scalar::random(&mut rng), Scalar::random(&mut rng)]))
        .collect::<Vec<Polynomial<Scalar>>>();

    // public
    let shares = private_polys
        .iter()
        .map(|poly| Shares::new(&mut rng, poly, &participants))
        .collect::<Vec<Shares>>();

    // public
    let verification_keys = shares
        .iter()
        .map(|share| share.verification_keys(&participants))
        .collect::<Vec<ShareVerificationKeys>>();

    // verify shares
    for (shvk_vec, share) in verification_keys.iter().zip(&shares) {
        for ((participant, shvk), esh) in participants.iter().zip(shvk_vec).zip(&share.esh_vec) {
            assert!(esh.verify(participant, &G2Affine::from(shvk)))
        }
    }
}

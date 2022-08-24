use crate::participant::Participant;
use crate::shares::{ShareVerificationKeys, Shares};
use agora_interpolate::Polynomial;
use bls::{G2Affine, Scalar};
use ff::Field;

// TODO
/*
pub struct Node {
    private_key: Scalar,
    private_poly: Polynomial<Scalar>,
    participant: Participant,
    share_map: HashMap<Scalar, Share>,
}

    // methods?
    shsk: Scalar, // share signing key
    shvk: G2Affine, // share verification key
*/

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

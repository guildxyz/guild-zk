use crate::participant::Participant;
use crate::shares::{Evaluations, Shares};
use agora_interpolate::Polynomial;
use bls::{G2Affine, Scalar};
use ff::Field;

#[test]
fn dkg_23() {
    let mut rng = rand_core::OsRng;
    let private_keys = (0..3)
        .into_iter()
        .map(|i| Scalar::random(&mut rng))
        .collect::<Vec<Scalar>>();

    let participants = private_keys
        .iter()
        .enumerate()
        .map(|(i, private_key)| Participant {
            id: Scalar::from(i as u64),
            pubkey: G2Affine::from(G2Affine::generator() * private_key),
        })
        .collect::<Vec<Participant>>();

    // specific because 2 coefficients are needed for a first order polynomial/participant
    let polys = participants
        .iter()
        .map(|_| Polynomial::new(vec![Scalar::random(&mut rng), Scalar::random(&mut rng)]))
        .collect::<Vec<Polynomial<Scalar>>>();

    // public
    let shares = polys
        .iter()
        .map(|poly| Shares::new(&mut rng, poly, &participants))
        .collect::<Vec<Shares>>();

    // public
    let evaluations = shares
        .iter()
        .map(|share| share.evaluations(&participants))
        .collect::<Vec<Evaluations>>();

    // verify pubkey_shares shares
    for (evals, share) in evaluations.iter().zip(&shares) {
        for ((participant, ev), enc) in participants.iter().zip(evals).zip(&share.encrypted_shares)
        {
            assert!(enc.verify(participant, &G2Affine::from(ev)))
        }
    }
}

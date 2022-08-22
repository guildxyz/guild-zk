use crate::participant::Participant;
use crate::shares::{Evaluations, Shares};
use agora_interpolate::Polynomial;
use bls::{G2Affine, Scalar};
use ff::Field;

#[test]
fn dkg_23() {
    let mut rng = rand_core::OsRng;
    let secret_keys = (0..3)
        .into_iter()
        .map(|i| Scalar::random(&mut rng))
        .collect::<Vec<Scalar>>();

    let participants = secret_keys
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

    // verify shares
    for (evals, share) in evaluations.iter().zip(&shares) {
        for ((participant, ev), enc) in participants.iter().zip(evals).zip(&share.encrypted_shares)
        {
            assert!(enc.verify(participant, &G2Affine::from(ev)))
        }
    }

    // decrypt shares with own private key
    let decrypted_shares = participants
        .iter()
        .zip(&secret_keys)
        .zip(&shares)
        .map(|((p, sk), sh)| {
            sh.encrypted_shares
                .iter()
                .map(|enc| enc.decrypt(p, sk))
                .collect::<Vec<Scalar>>()
        })
        .collect::<Vec<Vec<Scalar>>>();

    // TODO is this correct?
    let private_threshold_shares = decrypted_shares
        .iter()
        .map(|sh| {
            let p = dbg!(Polynomial::interpolate(
                &[participants[0].id, participants[1].id, participants[2].id],
                sh,
            ))
            .unwrap();
            p.coeffs()[0]
        })
        .collect::<Vec<Scalar>>();
    assert!(false);
}

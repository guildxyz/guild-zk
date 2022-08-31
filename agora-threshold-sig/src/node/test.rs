use super::*;
use bls::G1Projective;

#[test]
fn dkg_23() {
    let mut rng = rand_core::OsRng;
    let parameters = Parameters::new(3, 2);
    // spin up nodes
    let mut nodes = (0..parameters.nodes())
        .map(|_| Node::<Discovery>::new(parameters, Keypair::random(&mut rng)))
        .collect::<Vec<Node<Discovery>>>();
    // collect participants (iter and iter_mut does not work together)
    for i in 0..parameters.nodes() {
        for j in 0..parameters.nodes() {
            if i != j {
                let pubkey = nodes[j].pubkey();
                nodes[i].collect_participant(pubkey);
            }
        }
    }
    // generate partial shares
    let mut nodes = nodes
        .into_iter()
        .map(|node| Node::<ShareCollection>::try_from(node).unwrap())
        .collect::<Vec<Node<ShareCollection>>>();
    // publish and collect shares
    for i in 0..parameters.nodes() {
        for j in 0..parameters.nodes() {
            if i != j {
                let address = nodes[j].address();
                let share = nodes[j].publish_share();
                nodes[i].collect_share(address, share).unwrap();
            }
        }
    }
    for node in &nodes {
        assert_eq!(node.phase.participants.len(), parameters.nodes());
        assert_eq!(node.phase.shares.len(), parameters.nodes());
    }
    // verify collected shares
    let nodes = nodes
        .into_iter()
        .map(|node| Node::<Finalized>::try_from(node).unwrap())
        .collect::<Vec<Node<Finalized>>>();
    /*
    assert_eq!(node_0.phase.global_vk, node_1.phase.global_vk);
    assert_eq!(node_1.phase.global_vk, node_2.phase.global_vk);
    // sign message
    let msg = b"hello world";
    let signatures = vec![node_0.sign(msg), node_1.sign(msg), node_2.sign(msg)];
    // TODO this is ugly write loops and macros
    signatures[0].verify(msg, &node_0.verifying_key());
    signatures[1].verify(msg, &node_1.verifying_key());
    signatures[2].verify(msg, &node_2.verifying_key());
    // check global sig validity
    let global_poly = Polynomial::interpolate(
        &[node_0.address().as_scalar(), node_1.address().as_scalar()],
        &[
            G1Projective::from(signatures[0].inner()),
            G1Projective::from(signatures[1].inner()),
        ],
    )
    .unwrap();
    let global_sig = Signature::from(global_poly.coeffs()[0]);
    assert!(global_sig.verify(msg, &node_2.global_verifying_key()));

    let global_poly = Polynomial::interpolate(
        &[node_0.address().as_scalar(), node_2.address().as_scalar()],
        &[
            G1Projective::from(signatures[0].inner()),
            G1Projective::from(signatures[2].inner()),
        ],
    )
    .unwrap();
    let global_sig = Signature::from(global_poly.coeffs()[0]);
    assert!(global_sig.verify(msg, &node_1.global_verifying_key()));

    let global_poly = Polynomial::interpolate(
        &[node_1.address().as_scalar(), node_2.address().as_scalar()],
        &[
            G1Projective::from(signatures[1].inner()),
            G1Projective::from(signatures[2].inner()),
        ],
    )
    .unwrap();
    let global_sig = Signature::from(global_poly.coeffs()[0]);
    assert!(global_sig.verify(msg, &node_0.global_verifying_key()));

    let global_poly = Polynomial::interpolate(
        &[
            node_0.address().as_scalar(),
            node_1.address().as_scalar(),
            node_2.address().as_scalar(),
        ],
        &[
            G1Projective::from(signatures[0].inner()),
            G1Projective::from(signatures[1].inner()),
            G1Projective::from(signatures[2].inner()),
        ],
    )
    .unwrap();
    let global_sig = Signature::from(global_poly.coeffs()[0]);
    assert!(global_sig.verify(msg, &node_0.global_verifying_key()));
    */
}

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

    for node in nodes.iter().skip(1) {
        assert_eq!(nodes[0].global_verifying_key(), node.global_verifying_key());
    }
    // sign message and verify individual signatures
    let msg = b"hello world";
    let signatures = nodes.iter().map(|node| node.sign(msg)).collect::<Vec<Signature>>();
    
    for (node, signature) in nodes.iter().zip(&signatures) {
        assert!(signature.verify(msg, &node.verifying_key()));
    }

    let subset_iterator = nodes.iter().cycle();
    for i in 0..nodes.len() {
        let mut addr_scalars = Vec::<Scalar>::with_capacity(parameters.threshold());
        let mut sig_points = Vec::<G1Projective>::with_capacity(parameters.threshold());
        for j in parameters.threshold() {
            addr_scalars.push(subset_iterator.next().unwrap().address().as_scalar());
            sig_points.push(subset_iterator.next().unwrap().address().as_scalar());
        }

    }
    /*
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

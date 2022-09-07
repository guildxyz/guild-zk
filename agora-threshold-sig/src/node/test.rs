use super::*;
use bls::G1Projective;

#[test]
fn dkg_32() {
    let mut rng = rand_core::OsRng;
    let parameters = Parameters::new(3, 2);
    run(&mut rng, parameters);
}

#[test]
fn dkg_53() {
    let mut rng = rand_core::OsRng;
    let parameters = Parameters::new(5, 3);
    run(&mut rng, parameters);
}

fn run(rng: &mut rand_core::OsRng, parameters: Parameters) {
    // spin up nodes
    let mut nodes = (0..parameters.nodes())
        .map(|_| Node::<Discovery>::new(parameters, Keypair::random(rng)))
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
                let share = nodes[j].publish_share().unwrap();
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
    let signatures = nodes
        .iter()
        .map(|node| node.sign(msg))
        .collect::<Vec<Signature>>();

    for (node, signature) in nodes.iter().zip(&signatures) {
        assert!(signature.verify(msg, &node.verifying_key()));
    }

    // test t of n signature validity
    let mut subset_iterator = nodes.iter().zip(&signatures).cycle();
    for i in 0..nodes.len() {
        let mut addr_scalars = Vec::<Scalar>::with_capacity(parameters.threshold());
        let mut sig_points = Vec::<G1Projective>::with_capacity(parameters.threshold());
        for _ in 0..parameters.threshold() - 1 {
            // NOTE unwrap is fine because we cycle the iterator "endlessly"
            let (node, signature) = subset_iterator.next().unwrap();
            addr_scalars.push(node.address().as_scalar());
            sig_points.push(G1Projective::from(signature.inner()));

            // reject signature with not enough signers
            let global_poly = Polynomial::interpolate(&addr_scalars, &sig_points).unwrap();
            let global_sig = Signature::from(global_poly.coeffs()[0]);
            assert!(!global_sig.verify(msg, &nodes[i].global_verifying_key()));
        }
        // reaching threshold now
        let (node, signature) = subset_iterator.next().unwrap();
        addr_scalars.push(node.address().as_scalar());
        sig_points.push(G1Projective::from(signature.inner()));
        assert_eq!(addr_scalars.len(), parameters.threshold());
        assert_eq!(sig_points.len(), parameters.threshold());
        let global_poly = Polynomial::interpolate(&addr_scalars, &sig_points).unwrap();
        let global_sig = Signature::from(global_poly.coeffs()[0]);
        assert!(global_sig.verify(msg, &nodes[i].global_verifying_key()));
    }

    // test n out of n signature validity
    let mut addr_scalars = Vec::<Scalar>::with_capacity(parameters.nodes());
    let mut sig_points = Vec::<G1Projective>::with_capacity(parameters.nodes());

    for (node, signature) in nodes.iter().zip(&signatures) {
        addr_scalars.push(node.address().as_scalar());
        sig_points.push(G1Projective::from(signature.inner()));
    }

    let global_poly = Polynomial::interpolate(&addr_scalars, &sig_points).unwrap();
    let global_sig = Signature::from(global_poly.coeffs()[0]);
    assert!(global_sig.verify(msg, &nodes[0].global_verifying_key()));
}

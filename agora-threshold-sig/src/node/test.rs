use super::*;
use bls::G1Projective;

#[test]
fn dkg_23() {
    let mut rng = rand_core::OsRng;
    let parameters = Parameters::new(2, 3);
    let mut old_nodes = first_round(&mut rng, parameters);
    let old_global_vk = old_nodes[0].global_verifying_key(); // save for testing purposes
    let new_parameters = Parameters::new(3, 4);
    let mut new_node = Node::<Discovery>::new(new_parameters, Keypair::random(&mut rng));

    // collect participants
    for node in old_nodes.iter_mut() {
        new_node.collect_participant(node.pubkey());
        node.collect_participant(new_node.pubkey());
    }

    // old nodes initiate resharing and share collection
    let mut nodes = old_nodes
        .into_iter()
        .map(|node| {
            node.initiate_resharing(new_parameters)
                .unwrap()
                .initiate_share_collection()
        })
        .collect::<Vec<Node<ShareCollection>>>();

    // don't generate shares just wait for old node's shares
    let new_node = Node::<ShareCollection>::try_from(new_node).unwrap();
    // add new node to the node pool
    nodes.push(new_node);
    // publish and collect shares (new node will publish None)
    // TODO duplicate code (refractor into a function? But it's
    // tricky because of both & and &mut references to nodes)
    for i in 0..new_parameters.nodes() {
        for j in 0..new_parameters.nodes() {
            if i != j {
                let address = nodes[j].address();
                let share = nodes[j].publish_share();
                nodes[i].collect_share(address, share).unwrap();
            }
        }
    }
    // check that the new node didn't send a share
    for node in &nodes {
        assert_eq!(node.participants.len(), new_parameters.nodes());
        assert_eq!(node.phase.shares_map.len(), new_parameters.nodes());
    }
    // verify collected shares
    let nodes = nodes
        .into_iter()
        .map(|node| Node::<Finalized>::try_from(node).unwrap())
        .collect::<Vec<Node<Finalized>>>();

    for node in nodes.iter() {
        assert_eq!(old_global_vk, node.global_verifying_key());
    }
}

#[test]
fn dkg_35() {
    let mut rng = rand_core::OsRng;
    let parameters = Parameters::new(3, 5);
    let _og_nodes = first_round(&mut rng, parameters);
}

fn first_round(rng: &mut rand_core::OsRng, parameters: Parameters) -> Vec<Node<Finalized>> {
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
        .map(|node| {
            Node::<ShareGeneration>::try_from(node)
                .unwrap()
                .initiate_share_collection()
        })
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
        assert_eq!(node.participants.len(), parameters.nodes());
        assert_eq!(node.phase.shares_map.len(), parameters.nodes());
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

    nodes
}

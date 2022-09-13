use super::*;
use agora_interpolate::Polynomial;
use bls::{G1Projective, Scalar};

#[test]
fn dkg_23() {
    let mut rng = rand_core::OsRng;
    let parameters = Parameters::new(2, 3);
    let old_nodes = initial_round(&mut rng, parameters);
    test_signature_and_encryption(&mut rng, &parameters, &old_nodes);
    // resharing
    let parameters = Parameters::new(3, 4);
    let nodes = resharing(&mut rng, parameters, old_nodes);
    test_signature_and_encryption(&mut rng, &parameters, &nodes);
}

#[test]
fn dkg_35() {
    let mut rng = rand_core::OsRng;
    let parameters = Parameters::new(3, 5);
    let old_nodes = initial_round(&mut rng, parameters);
    test_signature_and_encryption(&mut rng, &parameters, &old_nodes);
    // resharing
    let parameters = Parameters::new(5, 7);
    let nodes = resharing(&mut rng, parameters, old_nodes);
    test_signature_and_encryption(&mut rng, &parameters, &nodes);
}

fn resharing(
    rng: &mut rand_core::OsRng,
    parameters: Parameters,
    mut old_nodes: Vec<Node<Finalized>>,
) -> Vec<Node<Finalized>> {
    let old_global_vk = old_nodes[0].global_verifying_key();
    let n = (parameters.nodes() as isize - old_nodes[0].parameters.nodes() as isize).unsigned_abs();
    let mut new_nodes = (0..n)
        .map(|_| Node::<Discovery>::new(parameters, Keypair::random(rng)))
        .collect::<Vec<Node<Discovery>>>();

    // collect participants
    for old_node in old_nodes.iter_mut() {
        for new_node in new_nodes.iter_mut() {
            new_node.collect_participant(old_node.pubkey());
            old_node.collect_participant(new_node.pubkey());
        }
    }
    // TODO put this in a function?
    // new participants also collect each other's public info
    for i in 0..new_nodes.len() {
        for j in 0..new_nodes.len() {
            if i != j {
                let pubkey = new_nodes[j].pubkey();
                new_nodes[i].collect_participant(pubkey);
            }
        }
    }

    // old nodes initiate resharing and share collection
    let mut nodes = old_nodes
        .into_iter()
        .map(|node| {
            node.initiate_resharing(parameters)
                .unwrap()
                .initiate_share_collection()
        })
        .collect::<Vec<Node<ShareCollection>>>();

    // don't generate shares just wait for old node's shares
    // add new nodes to the node pool
    for node in new_nodes
        .into_iter()
        .map(|node| Node::<ShareCollection>::try_from(node).unwrap())
    {
        nodes.push(node);
    }

    // publish and collect shares (new node will publish None)
    // TODO duplicate code (refractor into a function? But it's
    // tricky because of both & and &mut references to nodes)
    for i in 0..nodes.len() {
        for j in 0..nodes.len() {
            if i != j {
                if let Some(share) = nodes[j].publish_share() {
                    let address = nodes[j].address();
                    nodes[i].collect_share(address, share).unwrap();
                }
            }
        }
    }

    for node in &nodes {
        assert_eq!(node.participants.len(), parameters.nodes());
        assert_eq!(node.phase.shares_map.inner().len(), parameters.nodes() - n);
    }

    // verify collected shares
    let nodes = nodes
        .into_iter()
        .map(|node| Node::<Finalized>::try_from(node).unwrap())
        .collect::<Vec<Node<Finalized>>>();

    for node in nodes.iter() {
        assert_eq!(old_global_vk, node.global_verifying_key());
    }

    nodes
}

fn initial_round(rng: &mut rand_core::OsRng, parameters: Parameters) -> Vec<Node<Finalized>> {
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
    for i in 0..nodes.len() {
        for j in 0..nodes.len() {
            if i != j {
                if let Some(share) = nodes[j].publish_share() {
                    let address = nodes[j].address();
                    nodes[i].collect_share(address, share).unwrap();
                }
            }
        }
    }
    for node in &nodes {
        assert_eq!(node.participants.len(), parameters.nodes());
        assert_eq!(node.phase.shares_map.inner().len(), parameters.nodes());
    }
    // verify collected shares
    let nodes = nodes
        .into_iter()
        .map(|node| Node::<Finalized>::try_from(node).unwrap())
        .collect::<Vec<Node<Finalized>>>();

    for node in nodes.iter().skip(1) {
        assert_eq!(nodes[0].global_verifying_key(), node.global_verifying_key());
    }

    nodes
}

fn test_signature_and_encryption(
    rng: &mut rand_core::OsRng,
    parameters: &Parameters,
    nodes: &[Node<Finalized>],
) {
    // sign message and verify individual signatures
    let msg = b"hello world";
    let encryption = Encryption::new(rng, msg, nodes[0].global_verifying_key()).unwrap();
    let signatures = nodes
        .iter()
        .map(|node| node.sign(msg))
        .collect::<Vec<Signature>>();

    let decryption_shares = nodes
        .iter()
        .map(|node| node.decryption_share(&encryption))
        .collect::<Vec<G2Projective>>();

    for (node, signature) in nodes.iter().zip(&signatures) {
        assert!(signature.verify(msg, &node.verifying_key()));
    }

    // test t of n signature validity
    let mut subset_iterator = nodes
        .iter()
        .zip(&signatures)
        .zip(&decryption_shares)
        .cycle();
    for node in nodes {
        let mut addr_scalars = Vec::<Scalar>::with_capacity(parameters.threshold());
        let mut sig_points = Vec::<G1Projective>::with_capacity(parameters.threshold());
        let mut decryption_points = Vec::<G2Projective>::with_capacity(parameters.threshold());
        for _ in 0..parameters.threshold() - 1 {
            // NOTE unwrap is fine because we cycle the iterator "endlessly"
            let ((subset_node, signature), decryption_share) = subset_iterator.next().unwrap();
            addr_scalars.push(subset_node.address().as_scalar());
            sig_points.push(G1Projective::from(signature.inner()));
            decryption_points.push(*decryption_share);

            // reject signature with not enough signers
            let global_poly = Polynomial::interpolate(&addr_scalars, &sig_points).unwrap();
            let global_sig = Signature::from(global_poly.coeffs()[0]);
            assert!(!global_sig.verify(msg, &node.global_verifying_key()));
            let global_poly = Polynomial::interpolate(&addr_scalars, &decryption_points).unwrap();
            let decryption_key = global_poly.coeffs()[0];
            assert!(encryption
                .decrypt_with_pubkey(&decryption_key.into())
                .is_err());
        }
        // reaching threshold now
        let ((subset_node, signature), decryption_share) = subset_iterator.next().unwrap();
        addr_scalars.push(subset_node.address().as_scalar());
        sig_points.push(G1Projective::from(signature.inner()));
        decryption_points.push(*decryption_share);
        assert_eq!(addr_scalars.len(), parameters.threshold());
        assert_eq!(sig_points.len(), parameters.threshold());
        assert_eq!(decryption_points.len(), parameters.threshold());
        let global_poly = Polynomial::interpolate(&addr_scalars, &sig_points).unwrap();
        let global_sig = Signature::from(global_poly.coeffs()[0]);
        assert!(global_sig.verify(msg, &node.global_verifying_key()));
        let global_poly = Polynomial::interpolate(&addr_scalars, &decryption_points).unwrap();
        let decryption_key = global_poly.coeffs()[0];
        let decrypted = encryption
            .decrypt_with_pubkey(&decryption_key.into())
            .unwrap();
        assert_eq!(decrypted, msg);
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
    let global_poly = Polynomial::interpolate(&addr_scalars, &decryption_shares).unwrap();
    let decryption_key = global_poly.coeffs()[0];
    let decrypted = encryption
        .decrypt_with_pubkey(&decryption_key.into())
        .unwrap();
    assert_eq!(decrypted, msg);
}

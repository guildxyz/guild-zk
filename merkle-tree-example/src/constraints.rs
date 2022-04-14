use crate::common::*;
use crate::{Root, SimplePath};
use crate::pubkey::Pubkey;
use ark_crypto_primitives::crh::{TwoToOneCRH, TwoToOneCRHGadget, CRH};
use ark_crypto_primitives::merkle_tree::constraints::PathVar;
use ark_crypto_primitives::merkle_tree::Path;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::bits::ToBytesGadget;

use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::One;
use ark_groth16::{
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
};
use ark_std::test_rng;

// (You don't need to worry about what's going on in the next two type definitions,
// just know that these are types that you can use.)

/// The R1CS equivalent of the the Merkle tree root.
pub type RootVar = <TwoToOneHashGadget as TwoToOneCRHGadget<TwoToOneHash, ConstraintF>>::OutputVar;

/// The R1CS equivalent of the the Merkle tree path.
pub type SimplePathVar =
    PathVar<crate::MerkleConfig, LeafHashGadget, TwoToOneHashGadget, ConstraintF>;

////////////////////////////////////////////////////////////////////////////////

pub struct MerkleTreeVerification {
    // These are constants that will be embedded into the circuit
    pub leaf_crh_params: <LeafHash as CRH>::Parameters,
    pub two_to_one_crh_params: <TwoToOneHash as TwoToOneCRH>::Parameters,

    // These are the public inputs to the circuit.
    pub root: Root,

    // This is the private witness to the circuit.
    pub leaf: Option<Pubkey>,
    pub authentication_path: Option<SimplePath>,
}

impl ConstraintSynthesizer<ConstraintF> for MerkleTreeVerification {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<ConstraintF>,
    ) -> Result<(), SynthesisError> {
        // First, we allocate the public inputs
        let root = RootVar::new_input(ark_relations::ns!(cs, "root_var"), || Ok(&self.root))?;

        // Then, we allocate the public parameters as constants:
        let leaf_crh_params = LeafHashParamsVar::new_constant(cs.clone(), &self.leaf_crh_params)?;
        let two_to_one_crh_params =
            TwoToOneHashParamsVar::new_constant(cs.clone(), &self.two_to_one_crh_params)?;

        // Finally, we allocate our path as a private witness variable:
        let path_witness = if let Some(path_data) = self.authentication_path {
            SimplePathVar::new_witness(ark_relations::ns!(cs, "path_var"), || Ok(path_data))?
        } else {
            let leaf_sibling_hash =
                <LeafHash as ark_crypto_primitives::CRH>::evaluate(&self.leaf_crh_params, &[0_u8])
                    .unwrap();
            SimplePathVar::new_witness(ark_relations::ns!(cs, "path_var"), || {
                Ok(Path {
                    leaf_index: 1,
                    auth_path: vec![],
                    leaf_sibling_hash,
                })
            })?
        };

        let pubkey_vec = if let Some(pubkey) = self.leaf {
            pubkey.as_vec()
        } else {
            Pubkey{secret_key: [0;32]}.as_vec()
        };

        let mut leaf_bytes = vec![];
        for byte in pubkey_vec {
            let leaf_byte = UInt8::new_witness(ark_relations::ns!(cs, "leaf_var"), || Ok(byte))?;
            leaf_bytes.push(leaf_byte);
        }

        let is_member = path_witness.verify_membership(&leaf_crh_params, &two_to_one_crh_params, &root, &leaf_bytes.as_slice())?;

        is_member.enforce_equal(&Boolean::TRUE)?;

        Ok(())
    }
}

use std::time::{Duration, Instant};

#[test]
fn groth16_usage() {
    let mut rng = &mut test_rng();

    // First, let's sample the public parameters for the hash functions:
    let leaf_crh_params = <LeafHash as CRH>::setup(&mut rng).unwrap();
    let two_to_one_crh_params = <TwoToOneHash as TwoToOneCRH>::setup(&mut rng).unwrap();
    
    let mut leaves = Vec::with_capacity(262144);
    for i in 0..262144_u32 {
        let mut bytes = [0_u8; 32];
        let index_bytes = i.to_le_bytes();
        for j in 0..4 {
            bytes[j] = index_bytes[j];
        }
        leaves.push(Pubkey{secret_key: bytes});
    }
    let selected_leaf_idx = 42069 as usize;

    let start = Instant::now();

    // Create an instance of our circuit (with the witness)
    let tree = crate::SimpleMerkleTree::new(
        &leaf_crh_params,
        &two_to_one_crh_params,
        &leaves, // the i-th entry is the i-th leaf.
    )
    .unwrap();

    // First, let's get the root we want to verify against:
    let root = tree.root();

    let path = tree.generate_proof(0).unwrap(); // we're 0-indexing!

    let circuit = MerkleTreeVerification {
        // constants
        leaf_crh_params: leaf_crh_params.clone(),
        two_to_one_crh_params: two_to_one_crh_params.clone(),

        // public inputs
        root,

        // witness
        leaf: None,
        authentication_path: Some(path.clone()),
    };

    let mut total_setup = Duration::new(0, 0);
    let mut total_proving = Duration::new(0, 0);
    let mut total_verifying = Duration::new(0, 0);
    let rng = &mut test_rng();

    let params = { generate_random_parameters::<Bls12_381, _, _>(circuit, rng).unwrap() };

    // Prepare the verification key (for proof verification)
    let pvk = prepare_verifying_key(&params.vk);
    total_setup += start.elapsed();

    let start = Instant::now();
    let proof = {
        // This should be a proof for the membership of a leaf with value 9. Let's check that!
        let path = tree.generate_proof(selected_leaf_idx as usize).unwrap(); // we're 0-indexing!
    
        let circuit = MerkleTreeVerification {
            // constants
            leaf_crh_params,
            two_to_one_crh_params,

            // public inputs
            root,

            // witness
            leaf: Some(leaves[selected_leaf_idx].clone()),
            authentication_path: Some(path),
        };
        // Create a proof with our parameters.
        create_random_proof(circuit, &params, rng).unwrap()
    };
    total_proving += start.elapsed();

    // Check the proof
    let start = Instant::now();
    let inputs: Vec<_> = [root].to_vec();
    let r = verify_proof(&pvk, &proof, &inputs).unwrap();
    total_verifying += start.elapsed();

    println!("Prepare: {}", total_setup.as_millis());
    println!("Prove: {}", total_proving.as_millis());
    println!("Verify: {}", total_verifying.as_millis());

    assert!(r);
}

// Run this test via `cargo test --release test_merkle_tree`.
#[test]
fn merkle_tree_constraints_correctness() {
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;

    // Let's set up an RNG for use within tests. Note that this is *not* safe
    // for any production use.
    let mut rng = ark_std::test_rng();

    // First, let's sample the public parameters for the hash functions:
    let leaf_crh_params = <LeafHash as CRH>::setup(&mut rng).unwrap();
    let two_to_one_crh_params = <TwoToOneHash as TwoToOneCRH>::setup(&mut rng).unwrap();

    // Next, let's construct our tree.
    // This follows the API in https://github.com/arkworks-rs/crypto-primitives/blob/6be606259eab0aec010015e2cfd45e4f134cd9bf/src/merkle_tree/mod.rs#L156
    let tree = crate::SimpleMerkleTree::new(
        &leaf_crh_params,
        &two_to_one_crh_params,
        &[[1_u8; 32], [2_u8; 32], [9_u8; 32], [15_u8; 32]], // the i-th entry is the i-th leaf.
    )
    .unwrap();

    // Now, let's try to generate a membership proof for the 5th item, i.e. 9.
    let proof = tree.generate_proof(2).unwrap(); // we're 0-indexing!
                                                 // This should be a proof for the membership of a leaf with value 9. Let's check that!

    // First, let's get the root we want to verify against:
    let root = tree.root();

    let circuit = MerkleTreeVerification {
        // constants
        leaf_crh_params,
        two_to_one_crh_params,

        // public inputs
        root,

        // witness
        leaf: Some(Pubkey{secret_key: [9_u8; 32]}),
        authentication_path: Some(proof),
    };
    // First, some boilerplat that helps with debugging
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Next, let's make the circuit!
    let cs = ConstraintSystem::new_ref();
    circuit.generate_constraints(cs.clone()).unwrap();
    // Let's check whether the constraint system is satisfied
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(is_satisfied);
}

// Run this test via `cargo test --release test_merkle_tree_constraints_soundness`.
// This tests that a given invalid authentication path will fail.
#[test]
fn merkle_tree_constraints_soundness() {
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;

    // Let's set up an RNG for use within tests. Note that this is *not* safe
    // for any production use.
    let mut rng = ark_std::test_rng();

    // First, let's sample the public parameters for the hash functions:
    let leaf_crh_params = <LeafHash as CRH>::setup(&mut rng).unwrap();
    let two_to_one_crh_params = <TwoToOneHash as TwoToOneCRH>::setup(&mut rng).unwrap();

    // Next, let's construct our tree.
    // This follows the API in https://github.com/arkworks-rs/crypto-primitives/blob/6be606259eab0aec010015e2cfd45e4f134cd9bf/src/merkle_tree/mod.rs#L156
    let tree = crate::SimpleMerkleTree::new(
        &leaf_crh_params,
        &two_to_one_crh_params,
        &[1u32, 2, 9, 15], // the i-th entry is the i-th leaf.
    )
    .unwrap();

    // We just mutate the first leaf
    let second_tree = crate::SimpleMerkleTree::new(
        &leaf_crh_params,
        &two_to_one_crh_params,
        &[[4_u8; 32], [2_u8; 32], [9_u8; 32], [15_u8; 32]], // the i-th entry is the i-th leaf.
    )
    .unwrap();

    // Now, let's try to generate a membership proof for the 5th item, i.e. 9.
    let proof = tree.generate_proof(2).unwrap(); // we're 0-indexing!

    // But, let's get the root we want to verify against:
    let wrong_root = second_tree.root();

    let circuit = MerkleTreeVerification {
        // constants
        leaf_crh_params,
        two_to_one_crh_params,

        // public inputs
        root: wrong_root,

        // witness
        leaf: Some(Pubkey{secret_key: [9_u8; 32]}),
        authentication_path: Some(proof),
    };
    // First, some boilerplate that helps with debugging
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Next, let's make the constraint system!
    let cs = ConstraintSystem::new_ref();
    circuit.generate_constraints(cs.clone()).unwrap();
    // Let's check whether the constraint system is satisfied
    let is_satisfied = cs.is_satisfied().unwrap();
    // We expect this to fail!
    assert!(!is_satisfied);
}

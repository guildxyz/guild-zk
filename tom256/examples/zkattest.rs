use tom256::curve::{Secp256k1, Tom256k1};
use tom256::pedersen::PedersenCycle;
use tom256::proofs::{ZkAttestProof};
use tom256::parse::{ProofInput, ParsedProofInput};

use rand::rngs::StdRng;
use rand_core::SeedableRng;

fn main() {
    let mut rng = StdRng::from_seed([14; 32]);
    let pedersen_cycle = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

    let msg_hash = "0xb42062702a4acb9370edf5c571f2c7a6f448f8c42f3bfa59e622c1c064a94a14".to_string();
    let signature = "0xb2a7ff958cd78c8e896693b7b76550c8942d6499fb8cd621efb54909f9d51da02bfaadf918f09485740ba252445d40d44440fd810dbf8a9a18049157adcdaa8c1c".to_string();
    //let address = "0x2e3Eca6005eb4e30eA51692011612554586feaC9".to_string();
    let pubkey = "0x0418a30afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172fd55b3589c74fd4987b6004465afff77b039e631a68cdc7df9cd8cfd5cbe2887".to_string();

    let ring = vec![
        "0x0e3Eca6005eb4e30eA51692011612554586feaC9".to_string(),
        "0x1e3Eca6005eb4e30eA51692011612554586feaC9".to_string(),
        "0x2e3Eca6005eb4e30eA51692011612554586feaC9".to_string(),
        "0x3e3Eca6005eb4e30eA51692011612554586feaC9".to_string(),
        "0x4e3Eca6005eb4e30eA51692011612554586feaC9".to_string(),
    ];

    let index = 2;

    let proof_input = ProofInput {
        msg_hash,
        pubkey,
        signature,
        index,
        ring,
    };

    let parsed_input: ParsedProofInput::<Secp256k1, Tom256k1> = proof_input.try_into().unwrap();

    let address_parsed = parsed_input.ring[parsed_input.index];
    let address_committed = pedersen_cycle.cycle().commit(&mut rng, address_parsed);

    println!("{}", parsed_input.ring[parsed_input.index]);
    println!("{}", parsed_input.ring.len());
    println!("{}", parsed_input.msg_hash);
    println!("{}", parsed_input.signature.r);
    println!("{}", parsed_input.signature.s);
    println!("{}", parsed_input.index);

    let zkattest_proof = ZkAttestProof::<Secp256k1, Tom256k1>::construct(
        &mut rng,
        pedersen_cycle,
        address_committed,
        parsed_input,
    );

    let smth = zkattest_proof.unwrap();
    smth.verify(&mut rng).unwrap();
    /*
    assert!(zkattest_proof.is_ok());
    let zkattest_proof = zkattest_proof.unwrap();
    assert!(zkattest_proof.verify(&mut rng).is_ok());
    */
    
}

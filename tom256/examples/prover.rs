use rand_core::OsRng;
use structopt::StructOpt;
use tom256::curve::{Secp256k1, Tom256k1};
use tom256::parse::*;
use tom256::pedersen::PedersenCycle;
use tom256::proofs::ZkAttestProof;

use std::fs::File;
use std::io::Write;

#[derive(StructOpt)]
struct Opt {
    #[structopt(long, help = "array of public keys as string")]
    ring: String,
}

fn main() -> Result<(), String> {
    let opt = Opt::from_args();

    // generate pedersen parameters
    let mut rng = OsRng;
    let pedersen = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

    let ring: Ring = serde_json::from_str(&opt.ring).map_err(|e| e.to_string())?;

    let msg_hash = "0xb42062702a4acb9370edf5c571f2c7a6f448f8c42f3bfa59e622c1c064a94a14".to_string();
    let signature = "0xb2a7ff958cd78c8e896693b7b76550c8942d6499fb8cd621efb54909f9d51da02bfaadf918f09485740ba252445d40d44440fd810dbf8a9a18049157adcdaa8c1c".to_string();
    let pubkey = "0x0418a30afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172fd55b3589c74fd4987b6004465afff77b039e631a68cdc7df9cd8cfd5cbe2887".to_string();

    let proof_input = ProofInput {
        msg_hash,
        pubkey,
        signature,
        index: 7,
    };

    let parsed_input: ParsedProofInput<Secp256k1> = proof_input.try_into()?;
    let parsed_ring = parse_ring(ring)?;

    let zkattest_proof = ZkAttestProof::<Secp256k1, Tom256k1>::construct(
        &mut rng,
        pedersen,
        parsed_input,
        &parsed_ring,
    )?;

    let mut file = File::create("proof.json").map_err(|e| e.to_string())?;
    let proof = serde_json::to_string(&zkattest_proof).map_err(|e| e.to_string())?;
    write!(file, "{}", proof).map_err(|e| e.to_string())
}

/*
fn main() -> Result<(), String> {
    // parse ring from file
    let ring = read_ring_file();

    // generate pedersen parameters
    let mut rng = OsRng;
    let pedersen = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

    // dummy input
    let msg_hash = "0xb42062702a4acb9370edf5c571f2c7a6f448f8c42f3bfa59e622c1c064a94a14".to_string();
    let signature = "0xb2a7ff958cd78c8e896693b7b76550c8942d6499fb8cd621efb54909f9d51da02bfaadf918f09485740ba252445d40d44440fd810dbf8a9a18049157adcdaa8c1c".to_string();
    let pubkey = "0x0418a30afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172fd55b3589c74fd4987b6004465afff77b039e631a68cdc7df9cd8cfd5cbe2887".to_string();

    let proof_input = ProofInput {
        msg_hash,
        pubkey,
        signature,
        index,
        ring,
    };

    let parsed_input: ParsedProofInput<Secp256k1, Tom256k1> = proof_input.try_into()?;

    let zkattest_proof =
        ZkAttestProof::<Secp256k1, Tom256k1>::construct(&mut rng, pedersen_cycle, parsed_input)?;

    // write the proof to a file
    todo!();
}
*/
fn main() {}

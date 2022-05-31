use rand_core::OsRng;
use structopt::StructOpt;
use tom256::curve::{Secp256k1, Tom256k1};
use tom256::parse::*;
use tom256::proofs::ZkAttestProof;

#[derive(StructOpt)]
struct Opt {
    #[structopt(long, help = "zk ecdsa and membership proof")]
    proof: String,
    #[structopt(long, help = "array of public keys as string")]
    ring: String,
}

fn main() -> Result<(), String> {
    let mut rng = OsRng;
    let opt = Opt::from_args();
    let ring: Ring = serde_json::from_str(&opt.ring).map_err(|e| e.to_string())?;
    let parsed_ring = parse_ring(ring)?;

    let proof: ZkAttestProof<Secp256k1, Tom256k1> =
        serde_json::from_str(&opt.proof).map_err(|e| e.to_string())?;

    proof.verify(&mut rng, &parsed_ring)
}

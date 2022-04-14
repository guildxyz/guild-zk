use ark_crypto_primitives::commitment::pedersen::*;
use ark_crypto_primitives::commitment::CommitmentScheme;
use ark_ed_on_bls12_377::EdwardsProjective as Edwards;
use ark_std::UniformRand;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CommWindow;

impl Window for CommWindow {
    const WINDOW_SIZE: usize = 250;
    const NUM_WINDOWS: usize = 8;
}

fn main() {
    // play around with pedersen commitments
    let mut rng = ark_std::test_rng();
    let input = vec![5u8; 18];
    let parameters = Commitment::<Edwards, CommWindow>::setup(&mut rng).unwrap();
    let randomness = Randomness::rand(&mut rng);
    let commitment =
        Commitment::<Edwards, CommWindow>::commit(&parameters, &input, &randomness).unwrap();
    println!("{:?}", commitment);
}

struct EcdsaSignature {}

fn ecdsa_signature(msg: &[u8]) -> () {}

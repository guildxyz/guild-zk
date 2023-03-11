pub use ark_crypto_primitives::commitment::pedersen::Commitment;
pub use ark_crypto_primitives::commitment::pedersen::Parameters;
pub use ark_crypto_primitives::commitment::pedersen::Randomness;
pub use ark_crypto_primitives::commitment::CommitmentScheme;

#[derive(Clone, Copy)]
pub struct Window;

impl ark_crypto_primitives::commitment::pedersen::Window for Window {
    const WINDOW_SIZE: usize = 256;
    const NUM_WINDOWS: usize = 1;
}

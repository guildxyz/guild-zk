mod equality;
mod multiplication;

// NOTE these should be private and only point add and exp proofs should be
// public
pub use equality::EqualityProof;
pub use multiplication::MultiplicationProof;

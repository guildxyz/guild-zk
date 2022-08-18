use crate::encrypt::EncryptedShare;
use crate::participant::Participant;

use agora_interpolate::Polynomial;
use bls::{PulicKey, SecretKey, Scalar};


#[derive(Clone, Debug)]
pub struct Shares {
    coeff_pubkeys: Vec<PublicKey>,
    encrypted_shares: Vec<EncryptedShare>,
}

pub fn generate_shares(poly: &Polynomial, participants: &[Participant]) -> Shares {
    // TODO size checks?
    // t = poly.coeffs().len()
    // n = participants.len()
    let secret_shares = participants.idpoly.evaluate(

}

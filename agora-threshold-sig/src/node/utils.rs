use crate::address::Address;
use crate::share::{EncryptedShare, PublicShare};
use agora_interpolate::Polynomial;
use bls::{G2Affine, Scalar};
use ff::Field;

use std::collections::BTreeMap;

pub fn random_polynomial(threshold: usize, secret: Option<Scalar>) -> Polynomial<Scalar> {
    let mut private_coeffs = (0..threshold)
        .map(|_| Scalar::random(rand_core::OsRng))
        .collect::<Vec<Scalar>>();
    // in case of resharing, use the old secret
    if let Some(s) = secret {
        private_coeffs[0] = s;
    }
    Polynomial::new(private_coeffs)
}

pub fn generate_shares(
    participants: &BTreeMap<Address, G2Affine>,
    polynomial: &Polynomial<Scalar>,
) -> Vec<PublicShare> {
    participants
        .iter()
        .map(|(address, pubkey)| {
            let secret_share = polynomial.evaluate(address.as_scalar());
            let public_share = G2Affine::from(G2Affine::generator() * secret_share);
            let esh = EncryptedShare::new(
                &mut rand_core::OsRng,
                address.as_bytes(),
                pubkey,
                &secret_share,
            );
            PublicShare {
                vk: public_share,
                esh,
            }
        })
        .collect::<Vec<PublicShare>>()
}

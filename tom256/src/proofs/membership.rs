use super::utils::*;
use crate::arithmetic::{Modular, Point, Scalar};
use crate::pedersen::*;
use crate::{Curve, Cycle, U256};

use rand_core::{CryptoRng, RngCore};
use std::marker::PhantomData;

pub struct MembershipProof<CC, C> {
    cl: Vec<Point<CC>>,
    ca: Vec<Point<CC>>,
    cb: Vec<Point<CC>>,
    cd: Vec<Point<CC>>,
    fi: Vec<Scalar<CC>>,
    za: Vec<Scalar<CC>>,
    zb: Vec<Scalar<CC>>,
    zd: Scalar<CC>,
    base_curve: PhantomData<C>,
}

impl<CC: Cycle<C>, C: Curve> MembershipProof<CC, C> {
    const HASH_ID: &'static [u8] = b"membership-proof";

    pub fn construct<R: CryptoRng + RngCore>(
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<CC>,
        commitment_to_key: &PedersenCommitment<CC>,
        index: usize,
        // NOTE this is just the public address represented as a scalar (only
        // 160 bit, so it should fit unless C::PRIME_MODULUS is less than
        // 2^160)
        ring: &[Scalar<CC>],
    ) -> Result<Self, String> {
        let mut ring = ring.to_vec();
        let n = pad_ring_to_2n(&mut ring)?; // log2(ring.len())

        // random scalar storages
        let mut a_vec = Vec::<Scalar<CC>>::with_capacity(n);
        let mut l_vec = Vec::<Scalar<CC>>::with_capacity(n);
        let mut r_vec = Vec::<Scalar<CC>>::with_capacity(n);
        let mut s_vec = Vec::<Scalar<CC>>::with_capacity(n);
        let mut t_vec = Vec::<Scalar<CC>>::with_capacity(n);
        let mut rho_vec = Vec::<Scalar<CC>>::with_capacity(n);

        // commitment storages
        let mut ca = Vec::<Point<CC>>::with_capacity(n);
        let mut cb = Vec::<Point<CC>>::with_capacity(n);
        let mut cd = Vec::<Point<CC>>::with_capacity(n);
        let mut cl = Vec::<Point<CC>>::with_capacity(n);

        let mut omegas = Vec::<Scalar<CC>>::with_capacity(n);

        let mut tmp_index = index;
        for i in 0..n {
            l_vec.push(Scalar::new(U256::from_u64((tmp_index % 2) as u64)));
            tmp_index /= 2;
            a_vec.push(Scalar::random(rng));
            r_vec.push(Scalar::random(rng));
            s_vec.push(Scalar::random(rng));
            t_vec.push(Scalar::random(rng));
            rho_vec.push(Scalar::random(rng));

            cl.push(
                pedersen_generator
                    .commit_with_randomness(l_vec[i], r_vec[i])
                    .into_commitment(),
            );
            ca.push(
                pedersen_generator
                    .commit_with_randomness(a_vec[i], s_vec[i])
                    .into_commitment(),
            );
            cb.push(
                pedersen_generator
                    .commit_with_randomness(l_vec[i] * a_vec[i], t_vec[i])
                    .into_commitment(),
            );

            omegas.push(Scalar::new(U256::from_u64(i as u64)));
        }

        let mut poly_vals = Vec::<Scalar<CC>>::new();
        for omega in omegas.iter() {
            let mut f0j = Vec::<Scalar<CC>>::with_capacity(n);
            let mut f1j = Vec::<Scalar<CC>>::with_capacity(n);
            let mut ratio = Vec::<Scalar<CC>>::with_capacity(n);

            let mut product = Scalar::ONE;
            for j in 0..n {
                f0j[j] = &(Scalar::ONE - l_vec[j]) * omega - a_vec[j];
                f1j[j] = &l_vec[j] * omega + a_vec[j];
                ratio[j] = f1j[j] * f0j[j].inverse();
                product *= f0j[j];
            }

            let mut prod_vec = Vec::<Scalar<CC>>::new();
            prod_vec.push(product);

            for i in 0..n {
                let old_p_len = prod_vec.len();
                for j in 0..old_p_len {
                    prod_vec.push(ratio[i] * prod_vec[j]);
                }
            }

            let mut poly_val = Scalar::ZERO;
            for i in 0..n {
                poly_val += (ring[index] - ring[i]) * prod_vec[i];
            }
            poly_vals.push(poly_val);
        }

        let coeffs = interpolate(&omegas, &poly_vals)?;
        for i in 0..n {
            cd.push(
                pedersen_generator
                    .commit_with_randomness(coeffs[i], rho_vec[i])
                    .into_commitment(),
            );
        }

        // TODO hash points
        let challenge = Scalar::ZERO; // TODO
        let mut fi = Vec::<Scalar<CC>>::with_capacity(n);
        let mut za = Vec::<Scalar<CC>>::with_capacity(n);
        let mut zb = Vec::<Scalar<CC>>::with_capacity(n);
        let mut zd =
            commitment_to_key.randomness() * &challenge.pow(&Scalar::new(U256::from_u64(n as u64)));

        for i in 0..n {
            fi[i] = l_vec[i] * challenge + a_vec[i];
            za[i] = r_vec[i] * challenge + s_vec[i];
            zb[i] = r_vec[i] * (challenge - fi[i]) + t_vec[i];
            zd -= rho_vec[i] * challenge.pow(&Scalar::new(U256::from_u64(i as u64)));
        }

        Ok(Self {
            cl,
            ca,
            cb,
            cd,
            fi,
            za,
            zb,
            zd,
            base_curve: PhantomData,
        })
    }

    pub fn verify(
        &self,
        pedersen_generator: &PedersenGenerator<CC>,
        commitment: &Point<CC>,
        ring: &[Scalar<CC>],
    ) -> Result<(), String> {
        // TODO
        Ok(())
    }
}
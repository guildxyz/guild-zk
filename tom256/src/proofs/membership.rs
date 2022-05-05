use super::utils::*;
use crate::arithmetic::multimult::*;
use crate::arithmetic::{Modular, Point, Scalar};
use crate::pedersen::*;
use crate::utils::PointHasher;
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

            let mut prod_vec = vec![product];

            for r in ratio.iter().take(n) {
                let old_p_len = prod_vec.len();
                for j in 0..old_p_len {
                    prod_vec.push(r * &prod_vec[j]);
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

        if cl.len() != n || ca.len() != n || cb.len() != n || cd.len() != n {
            return Err("invalid commitment lengths".to_owned());
        }

        let challenge = Self::hash_commitments(&ca, &cb, &cd, &cl);
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

        if fi.len() != n || za.len() != n || zb.len() != n {
            return Err("invalid proof lengths".to_owned());
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

    pub fn verify<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<CC>,
        commitment: &Point<CC>,
        ring: &[Scalar<CC>],
    ) -> Result<(), String> {
        let mut ring = ring.to_vec();
        let n = pad_ring_to_2n(&mut ring)?; // log2(ring.len())

        let challenge = Self::hash_commitments(&self.ca, &self.cb, &self.cd, &self.cl);

        let mut multimult = MultiMult::new();
        multimult.add_known(Point::<CC>::GENERATOR);
        multimult.add_known(pedersen_generator.generator().clone());

        // NOTE unwraps here are fine because of length checks
        // at proof construction
        for i in 0..n {
            let mut rel_0 = Relation::new();
            let mut rel_1 = Relation::new();

            rel_0.insert(self.cl.get(i).unwrap().clone(), challenge);
            rel_0.insert(self.ca.get(i).unwrap().clone(), Scalar::ONE);
            rel_0.insert(Point::<CC>::GENERATOR, -self.fi[i]);
            rel_0.insert(pedersen_generator.generator().clone(), -self.za[i]);

            rel_1.insert(self.cl.get(i).unwrap().clone(), challenge - self.fi[i]);
            rel_1.insert(self.cb.get(i).unwrap().clone(), Scalar::ONE);
            rel_1.insert(pedersen_generator.generator().clone(), -self.zb[i]);

            rel_0.drain(rng, &mut multimult);
            rel_1.drain(rng, &mut multimult);
        }

        let mut total = Scalar::ZERO;
        for i in 0..ring.len() {
            let mut pix = Scalar::ONE;
            for j in 0..n {
                if i & (1 << j) == 0 {
                    pix *= challenge - self.fi[j]
                } else {
                    pix *= self.fi[j]
                }
            }
            total += ring[i] * pix;
        }

        let mut rel_final = Relation::new();
        // TODO
        Ok(())
    }

    fn hash_commitments(
        ca: &[Point<CC>],
        cb: &[Point<CC>],
        cd: &[Point<CC>],
        cl: &[Point<CC>],
    ) -> Scalar<CC> {
        let mut hasher = PointHasher::new(Self::HASH_ID);
        // NOTE we are assuming that all input slices have the same length
        // it is important to use this function in both `contruct` and `verify`
        for i in 0..ca.len() {
            hasher.insert_point(ca.get(i).unwrap());
            hasher.insert_point(cb.get(i).unwrap());
            hasher.insert_point(cd.get(i).unwrap());
            hasher.insert_point(cl.get(i).unwrap());
        }

        Scalar::<CC>::new(hasher.finalize())
    }
}

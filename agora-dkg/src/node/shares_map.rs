use crate::address::Address;
use crate::keypair::Keypair;
use crate::share::PublicShare;

use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
use thiserror::Error;
use zeroize::Zeroize;

use std::collections::BTreeMap;

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum SharesMapError {
    #[error("invalid share vector length {0}")]
    InvalidShareVectorLength(usize),
    #[error("invalid participants length {0}")]
    InvalidParticipantsLength(usize),
    #[error("index {0} is out of bounds")]
    InvalidIndex(usize),
    #[error("invalid shares collected")]
    InvalidShares,
    #[error("shares map is full")]
    SharesMapFull,
    #[error("{0} already provided shares")]
    SharesAlreadyProvided(Address),
    #[error("interpolation error")]
    InterpolationError(#[from] agora_interpolate::InterpolationError),
    #[error("keypair error")]
    KeypairError(#[from] crate::keypair::KeypairError),
    #[error("other: {0}")]
    Other(String),
}

/// Map accumulating all shares generated by individual participants. It can be
/// thought of as a public share matrix where each row consists of shares
/// generated by a single participant to other participants (and themselves).
/// Thus each column consists of shares generated for a single participant by
/// other participants.
///
/// It also contains the total number of participants which is used to check
/// that only valid length public share vectors are inserted in the map.
#[derive(Clone, Debug)]
pub struct SharesMap {
    map: BTreeMap<Address, Vec<PublicShare>>,
    share_vec_len: usize,
}

impl SharesMap {
    pub fn new(share_vec_len: usize) -> Self {
        Self {
            map: BTreeMap::new(),
            share_vec_len,
        }
    }

    pub fn map(&self) -> &BTreeMap<Address, Vec<PublicShare>> {
        &self.map
    }

    pub fn insert(
        &mut self,
        address: Address,
        share_vec: Vec<PublicShare>,
    ) -> Result<(), SharesMapError> {
        if share_vec.len() != self.share_vec_len {
            return Err(SharesMapError::InvalidShareVectorLength(share_vec.len()));
        } else if self.map.len() >= self.share_vec_len {
            return Err(SharesMapError::SharesMapFull);
        }

        if self.map.get(&address).is_some() {
            Err(SharesMapError::SharesAlreadyProvided(address))
        } else {
            self.map.insert(address, share_vec);
            Ok(())
        }
    }

    fn interpolated_shvks(
        &self,
        id_scalars: &[Scalar],
    ) -> Result<Vec<G2Projective>, SharesMapError> {
        let mut interpolated_shvks = Vec::<G2Projective>::with_capacity(self.share_vec_len);
        for i in 0..self.share_vec_len {
            let shvks = self
                .map
                .values()
                .map(|share_vec| share_vec[i].vk.into())
                .collect::<Vec<G2Projective>>();

            let poly = Polynomial::interpolate(id_scalars, &shvks)?;
            interpolated_shvks.push(poly.coeffs()[0]);
        }

        Ok(interpolated_shvks)
    }

    fn decrypted_shsks(
        &self,
        self_index: usize,
        self_address_bytes: &[u8; 32],
        self_privkey: &Scalar,
    ) -> Result<Vec<Scalar>, SharesMapError> {
        if self_index >= self.share_vec_len {
            return Err(SharesMapError::InvalidIndex(self_index));
        }

        let mut decrypted_shares_for_self = Vec::<Scalar>::with_capacity(self.map.len());
        for shares in self.map.values() {
            decrypted_shares_for_self.push(
                shares[self_index]
                    .esh
                    .decrypt(self_address_bytes, self_privkey),
            );
        }
        Ok(decrypted_shares_for_self)
    }

    fn verify_shares(&self, participants: &BTreeMap<Address, G2Affine>) -> bool {
        for share_vec in self.map.values() {
            for (address, share) in participants.keys().zip(share_vec) {
                if !share.esh.verify(address.as_bytes(), &share.vk) {
                    return false;
                }
            }
        }
        true
    }

    pub fn recover_keys(
        self,
        self_address: &Address,
        self_privkey: &Scalar,
        participants: &BTreeMap<Address, G2Affine>,
    ) -> Result<super::phase::Finalized, SharesMapError> {
        if participants.len() != self.share_vec_len {
            return Err(SharesMapError::InvalidParticipantsLength(
                participants.len(),
            ));
        } else if !self.verify_shares(participants) {
            return Err(SharesMapError::InvalidShares);
        }

        let share_id_scalars = self
            .map
            .keys()
            .map(|address| address.as_scalar())
            .collect::<Vec<Scalar>>();
        let mut self_index = None;
        let all_id_scalars = participants
            .keys()
            .enumerate()
            .map(|(i, address)| {
                if address == self_address {
                    self_index = Some(i);
                }
                address.as_scalar()
            })
            .collect::<Vec<Scalar>>();

        let self_index = self_index
            .ok_or_else(|| SharesMapError::Other("self index not found in storage".to_string()))?;
        let mut decrypted_shsks =
            self.decrypted_shsks(self_index, self_address.as_bytes(), self_privkey)?;
        let interpolated_shvks = self.interpolated_shvks(&share_id_scalars)?;

        let mut shsk_poly = Polynomial::interpolate(&share_id_scalars, &decrypted_shsks)?;
        let shsk = shsk_poly.coeffs()[0];
        shsk_poly.zeroize();
        decrypted_shsks.zeroize();

        let gshvk_poly = Polynomial::interpolate(&all_id_scalars, &interpolated_shvks)?;

        Ok(super::phase::Finalized {
            share_keypair: Keypair::new_checked(shsk, interpolated_shvks[self_index].into())?,
            global_vk: gshvk_poly.coeffs()[0].into(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::share::EncryptedShare;
    use bls::G1Affine;

    #[rustfmt::skip]
    const ADDRESSES: &[Address; 4] = &[
        Address::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        Address::new([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        Address::new([3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        Address::new([4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];

    const THRESHOLD: usize = 3;
    const START: usize = 2;

    // everything will be encrypted with this keypair
    fn test_keypair() -> Keypair {
        Keypair::new(Scalar::from(2))
    }

    fn test_polynomials(start: usize) -> (Polynomial<Scalar>, Polynomial<G2Projective>) {
        let mut scalars = Vec::<Scalar>::with_capacity(THRESHOLD);
        let mut points = Vec::<G2Projective>::with_capacity(THRESHOLD);
        for i in 0..THRESHOLD {
            let scalar = Scalar::from((start + i) as u64);
            let point = G2Affine::generator() * scalar;
            scalars.push(scalar);
            points.push(point);
        }
        (Polynomial::new(scalars), Polynomial::new(points))
    }

    fn test_participants() -> BTreeMap<Address, G2Affine> {
        ADDRESSES
            .iter()
            .map(|address| (*address, G2Affine::generator()))
            .collect()
    }

    // NOTE test map will look like this: These are the secret shares generated
    // by each node for themselves and others (by evaluating the polynomial at
    // the node identifier, i.e. the address as a scalar). The test map
    // actually consists of the public parts of these scalars, i.e. the
    // respective point on the curve and the secret share encrypted by the
    // recipient's public key
    //
    // Address        Polynomial  Node 1 shares  Node 2 shares  Node 3 shares  Node 4 shares
    //       1     2 + 3x + 4x^2              9             24             47             78
    //       2     5 + 6x + 7x^2             18             45             86            141
    //       3    8 + 9x + 10x^2             27             66            125            204
    //       4  11 + 12x + 13x^2             36             87            164            267
    fn build_test_map(keypair: &Keypair) -> SharesMap {
        let mut rng = rand_core::OsRng;
        let mut shares_map = SharesMap::new(ADDRESSES.len());

        for (i, address) in ADDRESSES.iter().enumerate() {
            let (private_poly, public_poly) = test_polynomials(START + i * THRESHOLD);
            let shares = ADDRESSES
                .iter()
                .map(|address| {
                    let public_share = public_poly.evaluate(address.as_scalar());
                    let private_share = private_poly.evaluate(address.as_scalar());
                    let esh = EncryptedShare::new(
                        &mut rng,
                        address.as_bytes(),
                        &keypair.pubkey(),
                        &private_share,
                    );
                    PublicShare {
                        vk: public_share.into(),
                        esh,
                    }
                })
                .collect::<Vec<PublicShare>>();
            shares_map.insert(*address, shares).unwrap();
        }

        shares_map
    }

    #[test]
    fn insert_shares() {
        fn default_share_vec(len: usize) -> Vec<PublicShare> {
            vec![
                PublicShare {
                    vk: G2Affine::generator(),
                    esh: EncryptedShare {
                        c: Scalar::zero(),
                        U: G2Affine::generator(),
                        V: G1Affine::generator(),
                    }
                };
                len
            ]
        }

        let n = 5;
        let mut sh_map = SharesMap::new(n);
        let mut address_bytes = [0u8; 32];
        sh_map
            .insert(Address::from(address_bytes), default_share_vec(n))
            .unwrap();
        address_bytes[0] += 1;
        sh_map
            .insert(Address::from(address_bytes), default_share_vec(n))
            .unwrap();
        assert_eq!(
            sh_map.insert(Address::from(address_bytes), default_share_vec(n)),
            Err(SharesMapError::SharesAlreadyProvided(Address::from(
                address_bytes
            )))
        );
        address_bytes[0] += 1;
        sh_map
            .insert(Address::from(address_bytes), default_share_vec(n))
            .unwrap();
        address_bytes[0] += 1;
        assert_eq!(
            sh_map.insert(Address::from(address_bytes), default_share_vec(n + 1)),
            Err(SharesMapError::InvalidShareVectorLength(n + 1))
        );
        assert_eq!(
            sh_map.insert(Address::from(address_bytes), default_share_vec(n - 1)),
            Err(SharesMapError::InvalidShareVectorLength(n - 1))
        );
        sh_map
            .insert(Address::from(address_bytes), default_share_vec(n))
            .unwrap();
        address_bytes[0] += 1;
        sh_map
            .insert(Address::from(address_bytes), default_share_vec(n))
            .unwrap();
        assert_eq!(
            sh_map.insert(Address::from(address_bytes), default_share_vec(n)),
            Err(SharesMapError::SharesMapFull)
        );
    }

    #[test]
    fn key_recovery_utils() {
        let test_keypair = test_keypair();
        let shares_map = build_test_map(&test_keypair);
        let participants = test_participants();
        shares_map.verify_shares(&participants);

        let scalars = participants
            .keys()
            .map(|address| address.as_scalar())
            .collect::<Vec<Scalar>>();
        let interpolated_shvks = shares_map.interpolated_shvks(&scalars).unwrap();
        let expected_secrets = [
            Scalar::from(0),
            Scalar::from(3),
            Scalar::from(8),
            Scalar::from(15),
        ];
        for (shvk, expected_secret) in interpolated_shvks.iter().zip(&expected_secrets) {
            let expected_public = G2Affine::generator() * expected_secret;
            assert_eq!(&expected_public, shvk)
        }

        #[rustfmt::skip]
        let expected_secret_evals_array = [
            [Scalar::from(9), Scalar::from(18), Scalar::from(27), Scalar::from(36)],
            [Scalar::from(24), Scalar::from(45), Scalar::from(66), Scalar::from(87)],
            [Scalar::from(47), Scalar::from(86), Scalar::from(125), Scalar::from(164)],
            [Scalar::from(78), Scalar::from(141), Scalar::from(204), Scalar::from(267)],
        ];

        for (i, (address, expected_secret_evals)) in ADDRESSES
            .iter()
            .zip(&expected_secret_evals_array)
            .enumerate()
        {
            let decrypted_shsks = shares_map
                .decrypted_shsks(i, address.as_bytes(), test_keypair.privkey())
                .unwrap();
            assert_eq!(&decrypted_shsks, expected_secret_evals);
        }
    }

    #[test]
    fn key_recovery() {
        let expected_secrets = [
            Scalar::from(0),
            Scalar::from(3),
            Scalar::from(8),
            Scalar::from(15),
        ];
        let test_keypair = test_keypair();
        let shares_map = build_test_map(&test_keypair);
        let participants = test_participants();
        for (address, expected_secret) in participants.keys().zip(&expected_secrets) {
            let keys = shares_map
                .clone()
                .recover_keys(address, test_keypair.privkey(), &participants)
                .unwrap();
            assert_eq!(expected_secret, keys.share_keypair.privkey());
        }
    }
}

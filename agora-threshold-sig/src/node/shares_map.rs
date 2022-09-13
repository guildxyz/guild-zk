use crate::address::Address;
use crate::keypair::Keypair;
use crate::share::PublicShare;

use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
use zeroize::Zeroize;

use std::collections::BTreeMap;

pub struct SharesMap(BTreeMap<Address, Vec<PublicShare>>);

impl SharesMap {
    pub fn new(inner: BTreeMap<Address, Vec<PublicShare>>) -> Self {
        Self(inner)
    }

    pub fn inner(&self) -> &BTreeMap<Address, Vec<PublicShare>> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut BTreeMap<Address, Vec<PublicShare>> {
        &mut self.0
    }

    fn interpolated_shvks(
        &self,
        participants: usize,
        id_scalars: &[Scalar],
    ) -> Result<Vec<G2Projective>, String> {
        let mut interpolated_shvks = Vec::<G2Projective>::with_capacity(participants);

        for i in 0..participants {
            let shvks = self
                .0
                .values()
                .map(|shares| shares[i].vk.into())
                .collect::<Vec<G2Projective>>();

            let poly = Polynomial::interpolate(id_scalars, &shvks).map_err(|e| e.to_string())?;
            interpolated_shvks.push(poly.coeffs()[0]);
        }

        Ok(interpolated_shvks)
    }

    fn decrypted_shsks(
        &self,
        self_index: usize,
        self_address_bytes: &[u8; 32],
        self_privkey: &Scalar,
    ) -> Vec<Scalar> {
        let mut decrypted_shares_for_self = Vec::<Scalar>::with_capacity(self.0.len());
        for shares in self.0.values() {
            decrypted_shares_for_self.push(
                shares[self_index]
                    .esh
                    .decrypt(self_address_bytes, self_privkey),
            );
        }
        decrypted_shares_for_self
    }

    pub fn recover_keys(
        self,
        self_address: &Address,
        self_privkey: &Scalar,
        participants: &BTreeMap<Address, G2Affine>,
    ) -> Result<super::phase::Finalized, String> {
        let share_id_scalars = self
            .0
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

        let self_index = self_index.ok_or_else(|| "self index not found in storage".to_string())?;
        let mut decrypted_shsks =
            self.decrypted_shsks(self_index, self_address.as_bytes(), self_privkey);
        let interpolated_shvks = self.interpolated_shvks(participants.len(), &share_id_scalars)?;

        let mut shsk_poly = Polynomial::interpolate(&share_id_scalars, &decrypted_shsks)
            .map_err(|e| e.to_string())?;
        let shsk = shsk_poly.coeffs()[0];
        shsk_poly.zeroize();
        decrypted_shsks.zeroize();

        let gshvk_poly = Polynomial::interpolate(&all_id_scalars, &interpolated_shvks)
            .map_err(|e| e.to_string())?;

        Ok(super::phase::Finalized {
            share_keypair: Keypair::new_checked(shsk, interpolated_shvks[self_index].into())?,
            global_vk: gshvk_poly.coeffs()[0].into(),
        })
    }
}

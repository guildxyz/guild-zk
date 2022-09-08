use crate::address::Address;
use crate::keypair::Keypair;
use crate::share::PublicShare;
use bls::G2Affine;
use std::collections::BTreeMap;

pub trait Phase {
    fn participants(&self) -> &BTreeMap<Address, G2Affine>;
    fn participants_mut(&mut self) -> &mut BTreeMap<Address, G2Affine>;
}

macro_rules! impl_phase_for {
    ($name:ty) => {
        impl Phase for $name {
            fn participants(&self) -> &BTreeMap<Address, G2Affine> {
                &self.participants
            }

            fn participants_mut(&mut self) -> &mut BTreeMap<Address, G2Affine> {
                &mut self.participants
            }
        }
    };
}

impl_phase_for!(Discovery);
impl_phase_for!(ShareCollection);
impl_phase_for!(Finalized);

pub struct Discovery {
    pub participants: BTreeMap<Address, G2Affine>,
}

pub struct ShareCollection {
    pub participants: BTreeMap<Address, G2Affine>,
    pub shares_map: BTreeMap<Address, Vec<PublicShare>>,
}

pub struct Finalized {
    pub participants: BTreeMap<Address, G2Affine>,
    pub share_keypair: Keypair,
    pub global_vk: G2Affine,
}

use ark_ec::models::short_weierstrass::SWCurveConfig;
use ark_std::{rand::Rng, UniformRand};

use super::multimult::MultiMult;
use super::pair::Pair;

pub struct Relation<C: SWCurveConfig> {
    pairs: Vec<Pair<C>>,
}

impl<C: SWCurveConfig> Default for Relation<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: SWCurveConfig> Relation<C> {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub fn insert(&mut self, pair: Pair<C>) {
        self.pairs.push(pair)
    }

    pub fn drain<R: Rng + ?Sized>(self, rng: &mut R, multimult: &mut MultiMult<C>) {
        let randomizer = C::ScalarField::rand(rng);
        for pair in self.pairs {
            multimult.insert_pair(pair * randomizer);
        }
    }
}

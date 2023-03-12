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

    pub fn insert_pair(&mut self, pair: Pair<C>) {
        self.pairs.push(pair)
    }

    pub fn drain<R: Rng + ?Sized>(self, rng: &mut R, multimult: &mut MultiMult<C>) {
        let randomizer = C::ScalarField::rand(rng);
        for pair in self.pairs {
            multimult.insert_pair(pair * randomizer);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_ec::models::short_weierstrass::{Affine, Projective, SWCurveConfig};
    use ark_ec::models::CurveConfig;
    use ark_ec::Group;
    use ark_secp256k1::Config as SecpConfig;
    use ark_secq256k1::Config as SecqConfig;
    use ark_serialize::CanonicalDeserialize;
    use ark_std::{
        rand::{rngs::StdRng, SeedableRng},
        UniformRand,
    };

    const SEED: u64 = 1234567890;

    macro_rules! test_relation {
        ($name:ident, $config:ident, $s:expr) => {
            #[test]
            fn $name() {
                let mut rng = StdRng::seed_from_u64(SEED);
                let mut rel = Relation::<$config>::new();

                let scalars: Vec<<$config as CurveConfig>::ScalarField> = (0..3)
                    .map(|_| <$config as CurveConfig>::ScalarField::rand(&mut rng))
                    .collect();

                let mut point = Projective::from($config::GENERATOR);
                for scalar in scalars.into_iter() {
                    point.double_in_place();
                    rel.insert_pair(Pair {
                        point: point.into(),
                        scalar,
                    });
                }

                let mut multimult = MultiMult::new();
                rel.drain(&mut rng, &mut multimult);
                let sum = multimult.evaluate();

                assert!(sum.is_on_curve());

                let decoded = hex::decode($s).unwrap();
                let expected = Affine::deserialize_compressed(&*decoded).unwrap();
                assert_eq!(sum, expected);
            }
        };
    }

    test_relation!(
        secp,
        SecpConfig,
        "4ba18a9ff87b498763098238948517ebc01d911511d6ae0be1bbe7e60fa307d780"
    );

    test_relation!(
        secq,
        SecqConfig,
        "d7fbb1f90d6243222575b5c8135318d513602c16bf658dc67d6916a2b4df451e00"
    );
}

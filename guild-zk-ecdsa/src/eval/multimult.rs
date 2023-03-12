use super::pair::{Known, Pair};

use ark_ec::models::short_weierstrass::{Affine, SWCurveConfig};
use ark_ec::models::CurveConfig;
use ark_ff::Field;

pub struct MultiMult<C: SWCurveConfig> {
    pub pairs: Vec<Pair<C>>,
    pub known: Vec<Known<C>>,
}

impl<C: SWCurveConfig> Default for MultiMult<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: SWCurveConfig> MultiMult<C> {
    pub fn new() -> Self {
        Self {
            pairs: Vec::new(),
            known: Vec::new(),
        }
    }

    pub fn insert_known(&mut self, point: Affine<C>) {
        if !self.known.iter().any(|known| known.point == point) {
            self.pairs.push(Pair {
                point,
                scalar: C::ScalarField::ZERO,
            });
            self.known.push(Known {
                point,
                index: self.pairs.len() - 1,
            });
        }
    }

    pub fn insert_pair(&mut self, pair: Pair<C>) {
        if let Some(element) = self.known.iter().find(|known| known.point == pair.point) {
            self.pairs[element.index].scalar += pair.scalar;
        } else {
            self.pairs.push(pair);
        }
    }

    pub fn evaluate(self) -> Affine<C> {
        if self.pairs.is_empty() {
            return Affine::<C>::identity();
        }
        if self.pairs.len() == 1 {
            return self.pairs[0].multiply();
        }

        let mut pairs_heap = super::heapify_vec(self.pairs);
        loop {
            // unwrap is fine here because peeking and pre-loop checks guarantee len is at least 1
            let a = pairs_heap.pop().unwrap();

            let c: Pair<C>;
            // If b_option is None -> the heap only has one element
            if let Some(mut b) = pairs_heap.peek_mut() {
                if b.scalar == <C as CurveConfig>::ScalarField::ZERO {
                    return a.multiply();
                }

                c = Pair {
                    point: a.point,
                    scalar: a.scalar - b.scalar,
                };
                let d = Pair {
                    point: (a.point + b.point).into(),
                    scalar: b.scalar,
                };

                *b = d;
            } else {
                return a.multiply();
            }

            if c.scalar != <C as CurveConfig>::ScalarField::ZERO {
                pairs_heap.push(c);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_ec::models::short_weierstrass::Projective;
    use ark_ec::Group;
    use ark_ff::One;
    use ark_secp256k1::Config as SecpConfig;
    use ark_std::{
        rand::{rngs::StdRng, SeedableRng},
        UniformRand,
    };
    use std::time::{Duration, Instant};

    const SEED: u64 = 1234567890;

    #[test]
    fn multimult_empty() {
        let multimult = MultiMult::<SecpConfig>::new();
        assert_eq!(multimult.evaluate(), Affine::<SecpConfig>::identity());
    }

    #[test]
    fn multimult_single() {
        let pair = Pair {
            point: SecpConfig::GENERATOR,
            scalar: <SecpConfig as CurveConfig>::ScalarField::one(),
        };

        let mut multimult = MultiMult::<SecpConfig>::new();
        multimult.insert_pair(pair);

        assert_eq!(multimult.evaluate(), pair.multiply());
    }

    #[test]
    fn multimult_multiple() {
        let mut rng = StdRng::seed_from_u64(SEED);

        let mut mm_time = Duration::new(0, 0);
        let mut normal_time = Duration::new(0, 0);

        let mut multimult = MultiMult::<SecpConfig>::new();
        let mut expected = Affine::identity();

        let summa_len = 10;
        let mut scalars = Vec::with_capacity(summa_len);
        for _ in 0..summa_len {
            scalars.push(<SecpConfig as CurveConfig>::ScalarField::rand(&mut rng));
        }

        for (i, scalar) in scalars.into_iter().enumerate() {
            let mut point = Projective::from(SecpConfig::GENERATOR);
            for _ in 0..i {
                point.double_in_place();
            }

            let now = Instant::now();
            let new_term = point * scalar;
            expected = (expected + new_term).into();
            normal_time += now.elapsed();

            multimult.insert_pair(Pair {
                point: point.into(),
                scalar,
            });
        }

        let now = Instant::now();
        let actual = multimult.evaluate();
        mm_time += now.elapsed();

        println!("Normal time: {:?}", normal_time);
        println!("Multimult time: {:?}", mm_time);

        assert_eq!(actual, expected);
    }
}

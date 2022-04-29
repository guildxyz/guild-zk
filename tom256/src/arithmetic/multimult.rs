use super::{Point, Scalar};

use crate::Curve;

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::binary_heap::BinaryHeap;

use rand_core::{CryptoRng, RngCore};

#[derive(Debug, Clone)]
pub struct Pair<C: Curve> {
    pub scalar: Scalar<C>,
    pub point: Point<C>,
}

pub struct Known<C: Curve> {
    point: Point<C>,
    index: usize,
}

pub struct MultiMult<C: Curve> {
    pairs: Vec<Pair<C>>,
    known: Vec<Known<C>>,
}

impl<C: Curve> Default for MultiMult<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Curve> MultiMult<C> {
    pub fn new() -> Self {
        Self {
            pairs: vec![],
            known: vec![],
        }
    }

    pub fn add_known(&mut self, pt: Point<C>) {
        if !self.known.iter().any(|known| known.point == pt) {
            self.known.push(Known {
                point: pt,
                index: self.known.len(),
            });
        }
    }

    pub fn insert(&mut self, point: Point<C>, scalar: Scalar<C>) {
        if let Some(element) = self.known.iter().find(|known| known.point == point) {
            self.pairs[element.index].scalar = self.pairs[element.index].scalar + scalar;
        } else {
            self.pairs.push(Pair::<C> { point, scalar });
        }
    }

    pub fn insert_pair(&mut self, pair: Pair<C>) {
        self.pairs.push(pair);
    }

    pub fn evaluate(self) -> Point<C> {
        if self.pairs.is_empty() {
            return Point::<C>::IDENTITY;
        }
        if self.pairs.len() == 1 {
            return self.pairs[0].point.scalar_mul(&self.pairs[0].scalar);
        }

        let mut pairs_heap = heapify_vec(self.pairs);
        loop {
            // unwrap is fine here because peeking and pre-loop checks guarantee len is at least 1
            let a = pairs_heap.pop().unwrap();

            let c: Pair<C>;
            // If b_option is None -> the heap only has one element
            if let Some(mut b) = pairs_heap.peek_mut() {
                if b.scalar == Scalar::<C>::ZERO {
                    return a.point.scalar_mul(&a.scalar);
                }

                c = Pair {
                    point: a.point.clone(),
                    scalar: a.scalar - b.scalar,
                };
                let d = Pair {
                    point: &a.point + &b.point,
                    scalar: b.scalar,
                };

                *b = d;
            } else {
                return a.point.scalar_mul(&a.scalar);
            }

            if c.scalar != Scalar::<C>::ZERO {
                pairs_heap.push(c);
            }
        }
    }
}

pub fn heapify_vec<T: Ord>(vec: Vec<T>) -> BinaryHeap<T> {
    vec.into_iter().collect()
}

pub struct Relation<C: Curve> {
    pairs: Vec<Pair<C>>,
}

impl<C: Curve> Default for Relation<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Curve> Relation<C> {
    pub fn new() -> Self {
        Self { pairs: vec![] }
    }

    pub fn insert(&mut self, point: Point<C>, scalar: Scalar<C>) {
        self.pairs.push(Pair { point, scalar })
    }
    pub fn drain<R: RngCore + CryptoRng>(self, rng: &mut R) -> MultiMult<C> {
        let randomizer = Scalar::<C>::random(rng);
        let mut multimult = MultiMult::<C>::new();
        for pair in self.pairs {
            multimult.insert(pair.point, pair.scalar * randomizer);
        }
        multimult
    }
}

// *************************************** TRAITS ***************************************** //

impl<C: Curve> PartialEq for Pair<C> {
    fn eq(&self, other: &Self) -> bool {
        self.scalar.eq(&other.scalar)
    }
}

impl<C: Curve> Eq for Pair<C> {}

impl<C: Curve> PartialOrd for Pair<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.scalar.partial_cmp(&other.scalar)
    }
}

impl<C: Curve> Ord for Pair<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.scalar.cmp(&other.scalar)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arithmetic::{FieldElement, Modular};
    use crate::Secp256k1;

    use bigint::U256;
    use rand_chacha::ChaChaRng;
    use rand_core::SeedableRng;

    use std::time::{Duration, Instant};

    type SecPoint = Point<Secp256k1>;
    type SecScalar = Scalar<Secp256k1>;

    // Get seeded CSPRNG for reproducible tests, usually for scalars
    // First 10 "random" scalars:
    // 83FEC693AC341A0F8F3F0E6A5B18AF130F3FBC2B06A00EA55743FA89E031CB5E
    // D125353892A829607AFCB23FEBB06E84C9745F1BF040BC6D1B64672A3B9148FD
    // F76C1FA7E623E38096A97FA0AF4D19CCE9A6D2CF62451F38D60245AED85E425F
    // 7FC351545F19EC3AECD29B4A5149A2FA56C0731CF34031E90EED16E2B78F1FA3
    // 1789EB7E7FC9BD1B0F2A7D6E9965DB607BE82D151E839727E4A28E37FB0E5F54
    // 14DBF7A81C7BFB869073A35D316923F137D530100A9981DC15206D2E14D61279
    // 9DD8AB9E34DCBD921BDE9156B0FDA2B845CD54F93FE0A6D0DA64B2D1F29457BA
    // A88D12AF2D2E7E71145176FFB5B954815EDF645B870FFBB5F4EDCC431380F116
    // 74BFB64E75EACFCDE34F877FDD224367AEC82A757D186B72FDECD37A43414231
    // 2F71C031C572EC94AC5F033233974EEA5E56698BA2DF9AF6F53D2C97A466D96A
    fn get_test_rng() -> ChaChaRng {
        ChaChaRng::from_seed([
            54, 1, 63, 153, 89, 49, 228, 122, 166, 230, 220, 138, 243, 90, 252, 212, 162, 48, 105,
            3, 140, 12, 169, 247, 176, 212, 208, 179, 38, 62, 94, 172,
        ])
    }

    #[test]
    fn multimult_empty() {
        let multimult = MultiMult::<Secp256k1>::new();
        assert_eq!(multimult.evaluate(), SecPoint::IDENTITY);
    }

    #[test]
    fn multimult_single() {
        let pt = SecPoint::GENERATOR;
        let scalar = SecScalar::ONE;

        let expected = pt.scalar_mul(&scalar);

        let mut multimult = MultiMult::<Secp256k1>::new();
        multimult.insert(pt, scalar);

        assert_eq!(multimult.evaluate(), expected);
    }

    #[test]
    fn multimult_multiple() {
        // Both work (true random / seeded random)
        // let mut rng = ChaChaRng::from_entropy();
        let mut rng = get_test_rng();

        let mut mm_time = Duration::new(0, 0);
        let mut normal_time = Duration::new(0, 0);

        let mut multimult = MultiMult::<Secp256k1>::new();
        let mut expected = SecPoint::IDENTITY;

        let summa_len = 10;
        let mut scalars = Vec::with_capacity(summa_len);
        for _ in 0..summa_len {
            scalars.push(SecScalar::random(&mut rng));
        }

        for (i, scalar) in scalars.iter().enumerate() {
            let mut pt = SecPoint::GENERATOR;
            for _ in 0..i {
                pt = pt.double();
            }

            let now = Instant::now();
            let new_term = pt.scalar_mul(scalar);
            expected = expected + new_term;
            normal_time += now.elapsed();

            multimult.insert(pt, *scalar);
        }

        let now = Instant::now();
        let actual = multimult.evaluate().into_affine();
        mm_time += now.elapsed();

        println!("Normal time: {:?}", normal_time);
        println!("Multimult time: {:?}", mm_time);

        assert_eq!(actual, expected);
    }

    #[test]
    fn relations() {
        let mut rng = get_test_rng();
        let mut rel = Relation::<Secp256k1>::new();

        let summa_len = 3;
        let mut scalars = Vec::with_capacity(summa_len);
        for _ in 0..summa_len {
            scalars.push(SecScalar::random(&mut rng));
        }

        for (i, scalar) in scalars.iter().enumerate() {
            let mut pt = SecPoint::GENERATOR;
            for _ in 0..i {
                pt = pt.double();
            }
            rel.insert(pt, *scalar);
        }

        let multimult = rel.drain(&mut rng);
        let sum = multimult.evaluate();
        let expected = SecPoint::new(
            FieldElement::new(U256::from_be_hex(
                "9913e57053c21be1383b08242483c1f245864bbd02b5f111b09dfbe9fe12ec7c",
            )),
            FieldElement::new(U256::from_be_hex(
                "5ccacf75bbae45598b952f580ba6906072efb914dd751a04182583884750d46a",
            )),
            FieldElement::ONE,
        );
        assert_eq!(sum, expected);
    }
}

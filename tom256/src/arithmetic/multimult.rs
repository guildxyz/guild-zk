use super::{FieldElement, Modular, Point, Scalar};

use crate::Curve;

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::binary_heap::BinaryHeap;

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

    pub fn evaluate(&mut self) -> Point<C> {
        if self.pairs.len() == 0 {
            return Point::<C>::IDENTITY;
        }
        if self.pairs.len() == 1 {
            return self.pairs[0].point.scalar_mul(&self.pairs[0].scalar);
        }

        let mut pairs_heap = heapify_vec(self.pairs.clone());
        //dbg!(&pairs_heap);
        let mut num_of_steps = 0;
        loop {
            num_of_steps += 1;
            //println!("{:?}", pairs_heap);
            // unwrap is fine here because peeking and pre-loop checks guarantee len is at least 1
            let a = pairs_heap.pop().unwrap();
            let c: Pair<C>;
            // If b_option is None -> the heap only has one element
            if let Some(mut b) = pairs_heap.peek_mut() {
                if b.scalar == Scalar::<C>::ZERO {
                    dbg!("Num of steps: {}", num_of_steps);
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
                dbg!("Num of steps: {}", num_of_steps);
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
    use crate::Secp256k1;

    use rand::rngs::StdRng;
    use rand_chacha::ChaChaRng;
    use rand_core::SeedableRng;

    use std::time::{Duration, Instant};

    type SecPoint = Point<Secp256k1>;
    type SecScalar = Scalar<Secp256k1>;

    #[test]
    fn multimult_empty() {
        let mut multimult = MultiMult::<Secp256k1>::new();
        assert_eq!(multimult.evaluate(), SecPoint::IDENTITY);
    }

    #[test]
    fn multimult_single_easy() {
        let pt = SecPoint::GENERATOR;
        let scalar = SecScalar::ONE;

        let expected = pt.scalar_mul(&scalar);

        let mut multimult = MultiMult::<Secp256k1>::new();
        multimult.insert(pt, scalar);

        assert_eq!(multimult.evaluate(), expected);
    }

    use bigint::{U256, Encoding,  Split};

    #[test]
    fn multimult_single_hard() {
        let mut rng = ChaChaRng::from_seed([54, 1, 63, 153, 89, 49, 228, 122, 166, 230, 220, 138, 243, 90, 252, 212, 162, 48, 105, 3, 140, 12, 169, 247, 176, 212, 208, 179, 38, 62, 94, 172]);
        //let mut rng = ChaChaRng::from_entropy();

        //println!("{:?}", rng.get_seed());

        let mut mm_time = Duration::new(0, 0);
        let mut normal_time = Duration::new(0, 0);

        let mut multimult = MultiMult::<Secp256k1>::new();
        let mut expected = SecPoint::IDENTITY;

        
        let scalars_r = vec![
            SecScalar::random(&mut rng),
            SecScalar::random(&mut rng),
            SecScalar::random(&mut rng),
            SecScalar::random(&mut rng),
            /*
            SecScalar::random(&mut rng),

            SecScalar::random(&mut rng),
            SecScalar::random(&mut rng),
            SecScalar::random(&mut rng),
            SecScalar::random(&mut rng),
            SecScalar::random(&mut rng),
            */
        ];
        
        
        
        let scalars = vec![
            SecScalar::new(U256::from_be_hex("83fec693ac341a0f8f3f0e6a5b18af130f3fbc2b06a00ea55743fa89e031cb5e")),
            SecScalar::new(U256::from_be_hex("d125353892a829607afcb23febb06e84c9745f1bf040bc6d1b64672a3b9148fd")),
            SecScalar::new(U256::from_be_hex("f76c1fa7e623e38096a97fa0af4d19cce9a6d2cf62451f38d60245aed85e425f")),
            SecScalar::new(U256::from_be_hex("7fc351545f19ec3aecd29b4a5149a2fa56c0731cf34031e90eed16e2b78f1fa3")),
        ];
        

        /*
        let scalars = vec![
            SecScalar::new(U256::from_be_hex("09E06E901916B0EFE2587BE65D2600EC231EB0A69B1F03F94A258621D0D8584B")),
            SecScalar::new(U256::from_be_hex("0DAD136B4C4628B9BADC425E7ED90FEB95D47D0B35EAC27C2E00AEA848DEB943")),
        ];
        */
        
        /*
        let scalars = vec![
            SecScalar::new(U256::from_be_hex("000000000000000000000000000000000000000000000000000000000000e031")),
            SecScalar::new(U256::from_be_hex("0000000000000000000000000000000000000000000000000000000000000a05")),
        ];
        */
        
        

        /*
        println!("Scalars:");
        println!("random bytes: {:?}", scalars_r[3].inner().to_be_bytes());
        println!("normal bytes: {:?}", scalars[3].inner().to_be_bytes());
        println!("random hex: {}", scalars_r[3].inner());
        println!("normal hex: {}", scalars[3].inner());
        println!("");
        */

        assert_eq!(scalars_r[0], scalars[0]);
        assert_eq!(scalars_r[1], scalars[1]);
        assert_eq!(scalars_r[2], scalars[2]);
        assert_eq!(scalars_r[3], scalars[3]);
        
        for i in 0..4 {
            let mut pt = SecPoint::GENERATOR;
            for _ in 0..i {
                pt = pt.double();
            }

            let pt_affine = pt.clone().into_affine();
            println!("\nPt {}", i);
            println!("x: {}", pt_affine.x().inner());
            println!("y: {}", pt_affine.y().inner());
            println!("z: {}", pt_affine.z().inner());

            //let scalar = SecScalar::random(&mut rng);
            let scalar = scalars[i];

            println!("\nScalar {}", i);
            println!("{}", scalar.inner());

            let now = Instant::now();
            let new_term = pt.scalar_mul(&scalar);
            let new_term_affine = new_term.clone().into_affine();
            println!("\nTerm {}", i);
            println!("x: {}", new_term_affine.x().inner());
            println!("y: {}", new_term_affine.y().inner());
            println!("z: {}", new_term_affine.z().inner());
            expected = expected + new_term;
            normal_time += now.elapsed();

            multimult.insert(pt, scalar);
        }

        let now = Instant::now();
        let actual = multimult.evaluate().into_affine();
        mm_time += now.elapsed();

        println!("\nActual");
        println!("x: {}", actual.x().inner());
        println!("y: {}", actual.y().inner());
        println!("z: {}", actual.z().inner());

        println!("\nExpected");
        let expected = expected.into_affine();
        println!("x: {}", expected.x().inner());
        println!("y: {}", expected.y().inner());
        println!("z: {}", expected.z().inner());

        println!("Normal time: {:?}", normal_time);
        println!("Multimult time: {:?}", mm_time);

        assert_eq!(actual, expected);
    }
}

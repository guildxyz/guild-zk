mod multimult;
mod pair;
mod relation;

pub use multimult::MultiMult;
pub use pair::Pair;
pub use relation::Relation;

use std::collections::binary_heap::BinaryHeap;

fn heapify_vec<T: std::cmp::Ord>(vec: Vec<T>) -> BinaryHeap<T> {
    vec.into_iter().collect()
}

/*

#[cfg(test)]
mod test {
    use super::*;
    use ark_secp256k1::Config as SecpConfig;
    use std::time::{Duration, Instant};

    //fn get_test_rng() -> StdRng {
    //    StdRng::from_seed([
    //        54, 1, 63, 153, 89, 49, 228, 122, 166, 230, 220, 138, 243, 90, 252, 212, 162, 48, 105,
    //        3, 140, 12, 169, 247, 176, 212, 208, 179, 38, 62, 94, 172,
    //    ])
    //}


    #[test]
    fn secp_relations() {
        let mut rng = get_test_rng();
        let mut rel = Relation::<Secp256k1>::new();

        let summa_len = 3;
        let mut scalars = Vec::with_capacity(summa_len);
        for _ in 0..summa_len {
            let scalar = SecScalar::random(&mut rng);
            scalars.push(scalar);
        }

        let mut pt: SecAffine = SecPoint::GENERATOR.into();
        for scalar in scalars.iter() {
            pt = pt.double().into();
            rel.insert(pt.into(), *scalar);
        }

        let mut multimult = MultiMult::new();
        rel.drain(&mut rng, &mut multimult);
        let sum = multimult.evaluate();
        let expected = SecPoint::new(
            FieldElement::new(U256::from_be_hex(
                "cf2de7b2e687085c14a39bb01457edfcac2cbadf67906de73d3251c6569d3089",
            )),
            FieldElement::new(U256::from_be_hex(
                "3fb1eb3907871809c1c46ba8d20b01798384cb6f9926d386c0456b7b01e4cbd5",
            )),
            FieldElement::ONE,
        );

        assert_eq!(sum, expected);
    }

    #[test]
    fn tom_relations() {
        let mut rng = get_test_rng();
        let mut rel = Relation::<Tom256k1>::new();

        let summa_len = 3;
        let mut scalars = Vec::with_capacity(summa_len);
        for _ in 0..summa_len {
            let scalar = TomScalar::random(&mut rng);
            scalars.push(scalar);
        }

        let mut pt: TomAffine = TomPoint::GENERATOR.into();
        for scalar in scalars.iter() {
            pt = pt.double().into();
            rel.insert(pt.into(), *scalar);
        }

        let mut multimult = MultiMult::new();
        rel.drain(&mut rng, &mut multimult);
        let sum = multimult.evaluate();
        let expected = TomPoint::new(
            FieldElement::new(U256::from_be_hex(
                "cccb91596829355ee5ab3682180025da88f0f93384149db1a2dca1c8c1011127",
            )),
            FieldElement::new(U256::from_be_hex(
                "f7b4bc3b8ccc4296ac43ce053b9fc375d889dae6b15511253a2431def4edb6d1",
            )),
            FieldElement::ONE,
        );

        assert_eq!(sum, expected);
    }
}
*/

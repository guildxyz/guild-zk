use crate::arithmetic::Scalar;
use crate::Curve;

pub fn pad_ring_to_2n<C: Curve>(ring: &mut Vec<Scalar<C>>) -> Result<usize, String> {
    // TODO ensure that the ring is not empty
    if ring.is_empty() {
        Err("empty ring".to_string())
    } else {
        let log_2_ring_len = ring.len().log2();
        let pow_2_ring_len = 2usize.pow(log_2_ring_len);
        // pow_2_ring_len is always less than or equal to keys.len()
        // because log2 always rounds down
        if ring.len() != pow_2_ring_len {
            for _ in 0..pow_2_ring_len * 2 - ring.len() {
                ring.push(ring[0])
            }
            Ok((log_2_ring_len + 1) as usize)
        } else {
            Ok(log_2_ring_len as usize)
        }
    }
}

pub fn eval_poly<C: Curve>(coeffs: &[Scalar<C>], x: Scalar<C>) -> Scalar<C> {
    let mut ret = Scalar::ZERO;
    for coeff in coeffs.iter().rev() {
        ret = *coeff + x * ret;
    }
    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arithmetic::Modular;
    use crate::{Tom256k1, U256};

    #[test]
    fn pad_ring() {
        let mut ring = Vec::<Scalar<Tom256k1>>::new();
        assert!(pad_ring_to_2n(&mut ring).is_err());
        ring.push(Scalar::ONE);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(0));
        assert_eq!(ring.len(), 1);
        ring.push(Scalar::ZERO);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(1));
        assert_eq!(ring.len(), 2);
        ring.push(Scalar::ZERO);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(2));
        assert_eq!(ring.len(), 4);
        assert_eq!(ring[3], Scalar::ONE);
        for _ in 0..5 {
            ring.push(Scalar::ZERO);
        }
        assert_eq!(ring.len(), 9);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(4));
        assert_eq!(ring.len(), 16);
        assert_eq!(ring[15], Scalar::ONE);
    }

    #[test]
    fn evaluate_polynomial() {
        // y = 2 * x^2 + 5 * x + 15
        let coeffs = vec![
            Scalar::<Tom256k1>::new(U256::from_u8(15)),
            Scalar::<Tom256k1>::new(U256::from_u8(5)),
            Scalar::<Tom256k1>::new(U256::from_u8(2)),
        ];

        let mut x = Scalar::new(U256::from_u8(3));
        assert_eq!(eval_poly(&coeffs, x).inner(), &U256::from_u8(48));
        x = Scalar::new(U256::from_u8(7));
        assert_eq!(eval_poly(&coeffs, x).inner(), &U256::from_u8(148));

        // y = 3 * x^4 + 4 * x^3 + 5 * x^2 + 9 * x + 10
        let coeffs = vec![
            Scalar::<Tom256k1>::new(U256::from_u8(10)), // c0
            Scalar::<Tom256k1>::new(U256::from_u8(9)), // c1
            Scalar::<Tom256k1>::new(U256::from_u8(5)), // c2
            Scalar::<Tom256k1>::new(U256::from_u8(4)), // c3
            Scalar::<Tom256k1>::new(U256::from_u8(3)), // c4
        ];
        x = Scalar::new(U256::from_u8(2));
        assert_eq!(eval_poly(&coeffs, x).inner(), &U256::from_u8(128));
    }
}

use crate::arithmetic::{Modular, Scalar};
use crate::{Curve, U256};

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

pub fn interpolate<C: Curve>(x: &[Scalar<C>], y: &[Scalar<C>]) -> Result<Vec<Scalar<C>>, String> {
    if x.len() != y.len() {
        return Err("input lengths not equal".to_string());
    }

    let n = x.len();

    let mut s = vec![Scalar::<C>::ZERO; n];
    let mut coeffs = vec![Scalar::<C>::ZERO; n];

    s.push(Scalar::ONE);
    s[n - 1] = -x[0];

    for i in 1..n {
        for j in n - 1 - i..n - 1 {
            s[j] = s[j] - x[i] * s[j + 1];
        }
        s[n - 1] -= x[i];
    }

    for i in 0..n {
        let mut phi = Scalar::ZERO;
        for j in (1..=n).rev() {
            phi = Scalar::new(U256::from_u64(j as u64)) * s[j] + x[i] * phi;
        }
        let ff = phi.inverse();
        let mut b = Scalar::ONE;
        for j in (0..n).rev() {
            coeffs[j] = coeffs[j] + b * ff * y[i];
            b = s[j] + x[i] * b;
        }
    }

    Ok(coeffs)
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
            Scalar::<Tom256k1>::new(U256::from_u8(9)),  // c1
            Scalar::<Tom256k1>::new(U256::from_u8(5)),  // c2
            Scalar::<Tom256k1>::new(U256::from_u8(4)),  // c3
            Scalar::<Tom256k1>::new(U256::from_u8(3)),  // c4
        ];
        x = Scalar::new(U256::from_u8(2));
        assert_eq!(eval_poly(&coeffs, x).inner(), &U256::from_u8(128));
    }

    #[test]
    fn interpolate_polynomial() {
        // constant polynomial (y = 53)
        let x = vec![Scalar::<Tom256k1>::new(U256::from_u8(3)); 1];
        let y = vec![Scalar::<Tom256k1>::new(U256::from_u8(53)); 1];
        let coeffs = interpolate(&x, &y).unwrap();
        assert_eq!(coeffs[0].inner(), &U256::from_u8(53));

        // simple first order polynomial (y = x)
        let x = vec![
            Scalar::<Tom256k1>::new(U256::from_u8(1)),
            Scalar::<Tom256k1>::new(U256::from_u8(2)),
            Scalar::<Tom256k1>::new(U256::from_u8(3)),
        ];

        let y = x.clone();
        let coeffs = interpolate(&x, &y).unwrap();
        assert_eq!(coeffs[0], Scalar::ZERO); // c_0
        assert_eq!(coeffs[1], Scalar::ONE); // c_1
        assert_eq!(coeffs[2], Scalar::ZERO); // c_2

        // first order polynomial (y = 22 * x + 7)
        let x = vec![
            Scalar::<Tom256k1>::new(U256::from_u8(2)),
            Scalar::<Tom256k1>::new(U256::from_u8(3)),
        ];
        let y = vec![
            Scalar::<Tom256k1>::new(U256::from_u8(51)),
            Scalar::<Tom256k1>::new(U256::from_u8(83)),
        ];
        let coeffs = interpolate(&x, &y).unwrap();
        // values taken from zkp js interpolate
        assert_eq!(
            coeffs[0].inner(),
            &U256::from_be_hex("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc22")
        );
        assert_eq!(coeffs[1].inner(), &U256::from_u8(0x20));

        // fourth order polynomial
        // y = x^4 + 0 * x^3 + 3 * x^2 + 2 * x + 14
        let x = vec![
            Scalar::<Tom256k1>::new(U256::from_u8(1)),
            Scalar::<Tom256k1>::new(U256::from_u8(2)),
            Scalar::<Tom256k1>::new(U256::from_u8(3)),
            Scalar::<Tom256k1>::new(U256::from_u8(4)),
            Scalar::<Tom256k1>::new(U256::from_u8(5)),
            Scalar::<Tom256k1>::new(U256::from_u8(6)),
        ];
        let y = vec![
            Scalar::<Tom256k1>::new(U256::from_u16(20)),
            Scalar::<Tom256k1>::new(U256::from_u16(46)),
            Scalar::<Tom256k1>::new(U256::from_u16(128)),
            Scalar::<Tom256k1>::new(U256::from_u16(326)),
            Scalar::<Tom256k1>::new(U256::from_u16(724)),
            Scalar::<Tom256k1>::new(U256::from_u16(1430)),
        ];
        let coeffs = interpolate(&x, &y).unwrap();
        assert_eq!(coeffs[0].inner(), &U256::from_u8(14)); // c0 (x^0)
        assert_eq!(coeffs[1].inner(), &U256::from_u8(2)); // c1 (x^1)
        assert_eq!(coeffs[2].inner(), &U256::from_u8(3)); // c2 (x^2)
        assert_eq!(coeffs[3].inner(), &U256::from_u8(0)); // c3 (x^3)
        assert_eq!(coeffs[4].inner(), &U256::from_u8(1)); // c4 (x^4)
        assert_eq!(coeffs[5].inner(), &U256::from_u8(0)); // c5 (x^5)
    }
}

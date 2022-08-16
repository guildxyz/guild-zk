use crate::arithmetic::Scalar;
use crate::curve::Curve;

pub fn pad_ring_to_2n<C: Curve>(ring: &mut Vec<Scalar<C>>) -> Result<usize, String> {
    // TODO ensure that the ring is not empty
    if ring.is_empty() {
        Err("empty ring".to_string())
    } else {
        let log_2_ring_len = ring.len().ilog2();
        let pow_2_ring_len = 2usize.pow(log_2_ring_len);
        // pow_2_ring_len is always less than or equal to keys.len()
        // because log2 always rounds down
        if ring.len() == pow_2_ring_len {
            Ok(log_2_ring_len as usize)
        } else {
            for _ in 0..pow_2_ring_len * 2 - ring.len() {
                ring.push(ring[0]);
            }
            Ok((log_2_ring_len + 1) as usize)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::curve::Tom256k1;

    type TomScalar = Scalar<Tom256k1>;

    #[test]
    fn pad_ring() {
        let mut ring = Vec::<TomScalar>::new();
        assert!(pad_ring_to_2n(&mut ring).is_err());
        ring.push(TomScalar::ONE);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(0));
        assert_eq!(ring.len(), 1);
        ring.push(TomScalar::ZERO);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(1));
        assert_eq!(ring.len(), 2);
        ring.push(TomScalar::ZERO);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(2));
        assert_eq!(ring.len(), 4);
        assert_eq!(ring[3], TomScalar::ONE);
        for _ in 0..5 {
            ring.push(TomScalar::ZERO);
        }
        assert_eq!(ring.len(), 9);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(4));
        assert_eq!(ring.len(), 16);
        assert_eq!(ring[15], TomScalar::ONE);
    }
}

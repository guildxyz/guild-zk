macro_rules! test_interpolate {
    ($t: ty) => {
        #[cfg(test)]
        mod test {
            use crate::{interpolate, Interpolate};
            use std::ops::Neg;

            type TestScalar = $t;

            #[test]
            fn interpolate_polynomial() {
                let x = vec![<TestScalar as Interpolate>::from_u64(3_u64); 3];
                let y = vec![<TestScalar as Interpolate>::from_u64(5_u64); 4];
                assert!(interpolate(&x, &y).is_err());

                // constant polynomial (y = 53)
                let x = vec![<TestScalar as Interpolate>::from_u64(3_u64); 1];
                let y = vec![<TestScalar as Interpolate>::from_u64(53_u64); 1];
                let coeffs = interpolate(&x, &y).unwrap();
                assert_eq!(coeffs[0], <TestScalar as Interpolate>::from_u64(53_u64));

                // simple first order polynomial (y = x)
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(1_u64),
                    <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestScalar as Interpolate>::from_u64(3_u64),
                ];

                let y = x.clone();
                let coeffs = interpolate(&x, &y).unwrap();
                assert_eq!(coeffs[0], <TestScalar as Interpolate>::zero()); // c_0
                assert_eq!(coeffs[1], <TestScalar as Interpolate>::one()); // c_1
                assert_eq!(coeffs[2], <TestScalar as Interpolate>::zero()); // c_2

                // first order polynomial (y = 32 * x - 13)
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestScalar as Interpolate>::from_u64(3_u64),
                ];
                let y = vec![
                    <TestScalar as Interpolate>::from_u64(51_u64),
                    <TestScalar as Interpolate>::from_u64(83_u64),
                ];
                let coeffs = interpolate(&x, &y).unwrap();
                assert_eq!(
                    coeffs[0],
                    <TestScalar as Interpolate>::from_u64(13_u64).neg()
                );
                assert_eq!(coeffs[1], <TestScalar as Interpolate>::from_u64(32_u64));

                // fourth order polynomial
                // y = x^4 + 0 * x^3 + 3 * x^2 + 2 * x + 14
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(1_u64),
                    <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestScalar as Interpolate>::from_u64(3_u64),
                    <TestScalar as Interpolate>::from_u64(4_u64),
                    <TestScalar as Interpolate>::from_u64(5_u64),
                    <TestScalar as Interpolate>::from_u64(6_u64),
                ];
                let y = vec![
                    <TestScalar as Interpolate>::from_u64(20_u64),
                    <TestScalar as Interpolate>::from_u64(46_u64),
                    <TestScalar as Interpolate>::from_u64(128_u64),
                    <TestScalar as Interpolate>::from_u64(326_u64),
                    <TestScalar as Interpolate>::from_u64(724_u64),
                    <TestScalar as Interpolate>::from_u64(1430_u64),
                ];
                let coeffs = interpolate(&x, &y).unwrap();
                assert_eq!(coeffs[0], <TestScalar as Interpolate>::from_u64(14_u64)); // c0 (x^0)
                assert_eq!(coeffs[1], <TestScalar as Interpolate>::from_u64(2_u64)); // c1 (x^1)
                assert_eq!(coeffs[2], <TestScalar as Interpolate>::from_u64(3_u64)); // c2 (x^2)
                assert_eq!(coeffs[3], <TestScalar as Interpolate>::from_u64(0_u64)); // c3 (x^3)
                assert_eq!(coeffs[4], <TestScalar as Interpolate>::from_u64(1_u64)); // c4 (x^4)
                assert_eq!(coeffs[5], <TestScalar as Interpolate>::from_u64(0_u64)); // c5 (x^5)
            }
        }
    };
}

pub(crate) use test_interpolate;

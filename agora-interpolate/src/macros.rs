macro_rules! test_polynomial {
    ($t: ty) => {
        #[cfg(test)]
        mod test {
            use crate::{Interpolate, InterpolationError, Polynomial};
            use std::ops::Neg;

            type TestScalar = $t;

            #[test]
            fn interpolate_and_evaluate() {
                // input slices of unequal length
                let x = vec![<TestScalar as Interpolate>::from_u64(3_u64); 3];
                let y = vec![<TestScalar as Interpolate>::from_u64(5_u64); 4];
                assert_eq!(
                    Polynomial::interpolate(&x, &y),
                    Err(InterpolationError::InvalidInputLengths(3, 4))
                );

                // constant polynomial (y = 53)
                let x = vec![<TestScalar as Interpolate>::from_u64(3_u64); 1];
                let y = vec![<TestScalar as Interpolate>::from_u64(53_u64); 1];
                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(
                    poly.coeffs()[0],
                    <TestScalar as Interpolate>::from_u64(53_u64)
                );
                assert_eq!(
                    poly.evaluate(<TestScalar as Interpolate>::from_u64(123456_u64),),
                    <TestScalar as Interpolate>::from_u64(53_u64),
                );
                assert_eq!(
                    poly.evaluate(<TestScalar as Interpolate>::from_u64(78910_u64),),
                    <TestScalar as Interpolate>::from_u64(53_u64),
                );

                // simple first order polynomial (y = x)
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(1_u64),
                    <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestScalar as Interpolate>::from_u64(3_u64),
                ];

                let y = x.clone();
                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(poly.coeffs()[0], <TestScalar as Interpolate>::zero()); // c_0
                assert_eq!(poly.coeffs()[1], <TestScalar as Interpolate>::one()); // c_1

                // first order polynomial (y = 32 * x - 13)
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestScalar as Interpolate>::from_u64(3_u64),
                ];
                let y = vec![
                    <TestScalar as Interpolate>::from_u64(51_u64),
                    <TestScalar as Interpolate>::from_u64(83_u64),
                ];
                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(
                    poly.coeffs()[0],
                    <TestScalar as Interpolate>::from_u64(13_u64).neg()
                );
                assert_eq!(
                    poly.coeffs()[1],
                    <TestScalar as Interpolate>::from_u64(32_u64)
                );

                assert_eq!(poly.evaluate(x[0]), y[0]);
                assert_eq!(poly.evaluate(x[1]), y[1]);
                assert_eq!(
                    poly.evaluate(<TestScalar as Interpolate>::from_u64(100_u64)),
                    <TestScalar as Interpolate>::from_u64(3187_u64)
                );

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
                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(
                    poly.coeffs()[0],
                    <TestScalar as Interpolate>::from_u64(14_u64)
                ); // c0 (x^0)
                assert_eq!(
                    poly.coeffs()[1],
                    <TestScalar as Interpolate>::from_u64(2_u64)
                ); // c1 (x^1)
                assert_eq!(
                    poly.coeffs()[2],
                    <TestScalar as Interpolate>::from_u64(3_u64)
                ); // c2 (x^2)
                assert_eq!(
                    poly.coeffs()[3],
                    <TestScalar as Interpolate>::from_u64(0_u64)
                ); // c3 (x^3)
                assert_eq!(
                    poly.coeffs()[4],
                    <TestScalar as Interpolate>::from_u64(1_u64)
                ); // c4 (x^4)

                assert_eq!(poly.evaluate(x[0]), y[0]);
                assert_eq!(poly.evaluate(x[1]), y[1]);
                assert_eq!(poly.evaluate(x[2]), y[2]);
                assert_eq!(poly.evaluate(x[3]), y[3]);
                assert_eq!(poly.evaluate(x[4]), y[4]);
            }
        }
    };
}

pub(crate) use test_polynomial;

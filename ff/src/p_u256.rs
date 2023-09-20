use std::str::FromStr;

use num_bigint::BigUint;
use primitive_types::U256;

use crate::errors::{Error, Result};

use utils::primality_test;

#[derive(Debug, Clone)]
pub struct FieldElement {
    pub num: U256,
    pub prime: U256,
}

impl FieldElement {
    pub fn from_u64(num: u64, prime: u64) -> Result<Self> {
        Self::new(U256::from(num), U256::from(prime))
    }

    pub fn from_str(num: &str, prime: &str) -> Result<Self> {
        Self::new(U256::from_str(num).unwrap(), U256::from_str(prime).unwrap())
    }

    pub fn from_u64_and_u256_prime(num: u64, prime: U256) -> Result<Self> {
        Self::new(U256::from(num), prime)
    }

    pub fn new(num: U256, prime: U256) -> Result<Self> {
        match primality_test::miller_rabin(BigUint::from_str(&prime.to_string()).unwrap()) {
            false => Err(Error::NotPrime),
            true => Ok(Self {
                num: num % prime,
                prime,
            }),
        }
    }

    ///    [a]. a(modp) + b(modp)
    ///    [b]. ( a(modp) + b(modp) )(modp)
    ///
    /// Result of [a]. can at most be 2p-2. If we pick a p value close to 2^64, this
    /// will clearly cause an overflow, given that we're operating in U256.
    ///
    /// |______________|===|*********|________|
    /// 0              p  U256       2p-2     U256
    ///  
    /// In that case, the result will be ******, and we'll need to add === to the
    /// result to make up for the overflow.
    pub fn add(&self, other: &Self) -> Result<Self> {
        Self::is_same_field(&self.prime, &other.prime)?;

        let (mut res, overflow) = self.num.overflowing_add(other.num);

        if overflow {
            res += (U256::MAX - self.prime) + 1;
        }

        res %= self.prime;

        Ok(Self {
            num: res,
            prime: self.prime,
        })
    }

    pub fn double(&self) -> Result<Self> {
        self.add(&self)
    }

    pub fn add_inv(&self) -> Result<Self> {
        Ok(Self {
            num: self.prime - self.num,
            prime: self.prime,
        })
    }

    /// (a(modp))-(b(modp)) (modp)  ==>  amodp + (-b)modp = amodp + (p-b)modp
    ///
    /// (a + p - b) is subject to overflows, but our addition function is already
    /// precautious against such situations
    pub fn sub(&self, other: &Self) -> Result<Self> {
        Self::is_same_field(&self.prime, &other.prime)?;

        let res = self.add(&other.add_inv()?)?;

        Ok(res)
    }

    /// Double & add algorithm
    ///
    /// Example: 5 * 45 = 5 * (101101)_2
    /// Iterate all bits of 45 starting from the LSB, and hold 2 aggregators:
    ///     base: this ticker will get doubled at each bit, unconditionally.
    ///             `base` will start from "5"
    ///     res:  we'll add `base` to this variable whenever the current bit is 1
    ///             `res` will start from "0"
    ///
    /// res = 5 + 20 + 40 + 160 = 225
    pub fn mul(&self, other: &Self) -> Result<Self> {
        Self::is_same_field(&self.prime, &other.prime)?;

        let mut other = other.num;
        if other == U256::zero() {
            return Ok(Self {
                num: U256::zero(),
                prime: self.prime,
            });
        }

        let mut base = self.clone();
        let mut res = Self {
            num: U256::zero(),
            prime: self.prime,
        };

        while other != U256::zero() {
            if other & U256::one() == U256::one() {
                res = res.add(&base)?;
            }
            base = base.double()?;
            other >>= 1;
        }

        Ok(res)
    }

    pub fn sq(&self) -> Result<Self> {
        self.mul(&self)
    }

    // Uses Fermat's little
    pub fn mul_inv(&self) -> Result<Self> {
        self.exp(&(&self.prime - 2))
    }

    pub fn div(&self, other: &Self) -> Result<Self> {
        let other_inv = other.mul_inv()?;
        let res = self.mul(&other_inv)?;
        Ok(res)
    }

    // Square & add algorithm
    pub fn exp(&self, exp: &U256) -> Result<Self> {
        // Use fermat's little theorem
        let mut exp = *exp % (self.prime - 1);

        if exp == U256::zero() {
            return Ok(Self {
                num: U256::zero(),
                prime: self.prime,
            });
        }

        let mut base = self.clone();
        let mut res = Self {
            num: U256::one(),
            prime: self.prime,
        };

        while exp != U256::zero() {
            if exp & U256::one() == U256::one() {
                res = res.mul(&base)?;
            }
            base = base.sq()?;
            exp >>= 1;
        }

        return Ok(res);
    }

    fn is_same_field(p_1: &U256, p_2: &U256) -> Result<()> {
        if *p_1 != *p_2 {
            return Err(Error::DifferentFields);
        }
        Ok(())
    }
}

impl Eq for FieldElement {}
impl PartialEq for FieldElement {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num && self.prime == other.prime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_err_not_a_prime() {
        let err = FieldElement::from_u64(17, 21).unwrap_err();
        assert_eq!(err, Error::NotPrime);
    }

    #[test]
    fn new_1() {
        let prime = U256::from(23);
        let num = U256::from(7871238);
        let a = FieldElement::new(num, prime).unwrap();
        assert_eq!(
            a,
            FieldElement {
                num: U256::from(17),
                prime
            }
        );
    }

    #[test]
    fn cmp_neq_1() {
        let a = FieldElement::from_u64(17, 23).unwrap();
        let b = FieldElement::from_u64(16, 23).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn cmp_neq_2() {
        let c = FieldElement::from_u64(17, 23).unwrap();
        let d = FieldElement::from_u64(17, 29).unwrap();
        assert_ne!(c, d);
    }

    #[test]
    fn eq_1() {
        let a = FieldElement::from_u64(17, 23).unwrap();
        let b = FieldElement::from_u64(17, 23).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn add_0() {
        let p = "0xB";

        let a = FieldElement::from_str("0xBD", &p).unwrap();
        let b = FieldElement::from_str("0x2B", &p).unwrap();

        let r = a.add(&b).unwrap();

        assert_eq!(
            r,
            FieldElement::from_str(
                "0000000000000000000000000000000000000000000000000000000000000001",
                &p
            )
            .unwrap()
        );
    }

    #[test]
    fn add_1() {
        let p = "0xf9cd";

        let a = FieldElement::from_str("0xa167f055ff75c", &p).unwrap();
        let b = FieldElement::from_str("0xacc457752e4ed", &p).unwrap();

        let r = a.add(&b).unwrap();

        assert_eq!(
            r,
            FieldElement::from_str(
                "0000000000000000000000000000000000000000000000000000000000006bb0",
                &p
            )
            .unwrap()
        );
    }

    #[test]
    fn add_2() {
        let p = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F";

        let a = FieldElement::from_str(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2E",
            &p,
        )
        .unwrap();
        let b = FieldElement::from_str(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2E",
            &p,
        )
        .unwrap();

        let r = a.add(&b).unwrap();

        assert_eq!(
            r,
            FieldElement::from_str(
                "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d",
                &p
            )
            .unwrap()
        );
    }

    #[test]
    fn add_3() {
        let a = FieldElement::from_u64(17, 797).unwrap();
        let b = FieldElement::from_u64(17, 859).unwrap();

        let err = a.add(&b).unwrap_err();
        assert_eq!(err, Error::DifferentFields);
    }

    #[test]
    fn add_4() {
        let prime: U256 = U256::from(859);
        let num1: U256 = U256::from(17);
        let num2: U256 = U256::from(2222223); // 849 mod 859

        let a = FieldElement::new(num1, prime).unwrap();
        let b = FieldElement::new(num2, prime).unwrap();

        let res = a.add(&b).unwrap();
        assert_eq!(
            res,
            FieldElement {
                prime,
                num: U256::from(7)
            }
        );
    }

    #[test]
    fn sub_err_different_primes() {
        let a = FieldElement::from_u64(17, 797).unwrap();
        let b = FieldElement::from_u64(17, 859).unwrap();

        let err = a.add(&b).unwrap_err();
        assert_eq!(err, Error::DifferentFields);
    }

    #[test]
    fn sub_1() {
        let prime: U256 = U256::from(859);
        let num1: U256 = U256::from(17);
        let num2: U256 = U256::from(2222223); // 849 mod 859

        let a = FieldElement::new(num1, prime).unwrap();
        let b = FieldElement::new(num2, prime).unwrap();

        let res = a.sub(&b).unwrap();
        assert_eq!(
            res,
            FieldElement {
                prime,
                num: U256::from(27)
            }
        );
    }

    #[test]
    fn mul_1() {
        let prime: U256 = U256::from(859);
        let num1: U256 = U256::from(17);
        let num2: U256 = U256::from(2222223); // 849 mod 859

        let expected_result: U256 = U256::from(689);

        let a = FieldElement::new(num1, prime).unwrap();
        let b = FieldElement::new(num2, prime).unwrap();

        let res = a.mul(&b).unwrap();
        assert_eq!(
            res,
            FieldElement {
                prime,
                num: expected_result
            }
        );
    }

    #[test]
    fn exp_1() {
        let prime = U256::from(97);
        let a = FieldElement::new(U256::from(3), prime).unwrap();
        let res = a.exp(&U256::from(4)).unwrap();
        assert_eq!(
            res,
            FieldElement {
                prime,
                num: U256::from(81)
            }
        );
    }

    #[test]
    fn exp_2() {
        let prime = U256::from(97);
        let a = FieldElement::new(U256::one(), prime).unwrap();
        let res = a.exp(&U256::from(326423784)).unwrap();
        assert_eq!(
            res,
            FieldElement {
                prime,
                num: U256::one()
            }
        );
    }

    #[test]
    fn exp_3() {
        let prime = U256::from_str("0xFFFFFFFFFFFFFFC5").unwrap();
        let a = FieldElement::new(U256::from(2), prime).unwrap();
        let res = a.exp(&U256::from(35)).unwrap();
        assert_eq!(
            res,
            FieldElement {
                prime,
                num: U256::from_str("0x800000000").unwrap()
            }
        );
    }

    #[test]
    fn test_div_ez() {
        let prime = 19;
        let a = FieldElement::from_u64(2, prime).unwrap();
        let b = FieldElement::from_u64(7, prime).unwrap();
        let c = FieldElement::from_u64(3, prime).unwrap();

        assert_eq!(a.div(&b).unwrap(), c);
    }

    #[test]
    fn test_div_hard() {
        let prime = 19;
        let a = FieldElement::from_u64(2, prime).unwrap();
        let b = FieldElement::from_u64(7, prime).unwrap();
        let c = FieldElement::from_u64(3, prime).unwrap();

        assert_eq!(a.div(&b).unwrap(), c);
    }
}

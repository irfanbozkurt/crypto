use num_bigint::BigUint;

use crate::errors::{Error, Result};

use utils::primality_test;

#[derive(Debug, Clone)]
pub struct U64FieldElement {
    pub num: u64,
    pub prime: u64,
}

impl U64FieldElement {
    const ZERO: u64 = 0;
    const ONE: u64 = 1;

    pub fn new(num: u64, prime: u64) -> Result<Self> {
        let is_prime = primality_test::miller_rabin(BigUint::from(prime));
        if !is_prime {
            return Err(Error::NotPrime);
        }

        Ok(Self {
            num: num % prime,
            prime,
        })
    }

    ///    [a]. a(modp) + b(modp)
    ///    [b]. ( a(modp) + b(modp) )(modp)
    ///
    /// Result of [a]. can at most be 2p-2. If we pick a p value close to 2^64, this
    /// will clearly cause an overflow, given that we're operating in u64.
    ///
    /// |______________|===|*********|________|
    /// 0              p  u64       2p-2     u64
    ///  
    /// In that case, the result will be ******, and we'll need to add === to the
    /// result to make up for the overflow.
    pub fn add(&self, other: &Self) -> Result<Self> {
        Self::is_same_field(&self.prime, &other.prime)?;

        let (mut res, overflow) = self.num.overflowing_add(other.num);

        if overflow {
            res += u64::MAX - self.prime;
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

    pub fn neg(&self) -> Result<Self> {
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

        let res = self.add(&other.neg()?)?;

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
        if other == Self::ZERO {
            return Ok(Self {
                num: 0,
                prime: self.prime,
            });
        }

        let mut base = self.clone();
        let mut res = Self {
            num: 0,
            prime: self.prime,
        };

        while other != Self::ZERO {
            if other & 1 == 1 {
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
    pub fn exp(&self, exp: &u64) -> Result<Self> {
        // Use fermat's little theorem
        let mut exp = *exp % (self.prime - 1);

        if exp == Self::ZERO {
            return Ok(Self {
                num: Self::ZERO,
                prime: self.prime,
            });
        }

        let mut base = self.clone();
        let mut res = Self {
            num: Self::ONE,
            prime: self.prime,
        };

        while exp != Self::ZERO {
            if exp & 1 == 1 {
                res = res.mul(&base)?;
            }
            base = base.sq()?;
            exp >>= 1;
        }

        return Ok(res);
    }

    fn is_same_field(p_1: &u64, p_2: &u64) -> Result<()> {
        if *p_1 != *p_2 {
            return Err(Error::DifferentFields);
        }
        Ok(())
    }
}

impl Eq for U64FieldElement {}
impl PartialEq for U64FieldElement {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num && self.prime == other.prime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_err_not_a_prime() {
        let err = U64FieldElement::new(17, 21).unwrap_err();
        assert_eq!(err, Error::NotPrime);
    }

    #[test]
    fn new_1() {
        let prime = 23;
        let num = 7871238;
        let a = U64FieldElement::new(num, prime).unwrap();
        assert_eq!(a, U64FieldElement { num: 17, prime });
    }

    #[test]
    fn cmp_neq_1() {
        let a = U64FieldElement::new(17, 23).unwrap();
        let b = U64FieldElement::new(16, 23).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn cmp_neq_2() {
        let c = U64FieldElement::new(17, 23).unwrap();
        let d = U64FieldElement::new(17, 29).unwrap();
        assert_ne!(c, d);
    }

    #[test]
    fn eq_1() {
        let a = U64FieldElement::new(17, 23).unwrap();
        let b = U64FieldElement::new(17, 23).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn add_err_different_primes() {
        let a = U64FieldElement::new(17, 797).unwrap();
        let b = U64FieldElement::new(17, 859).unwrap();

        let err = a.add(&b).unwrap_err();
        assert_eq!(err, Error::DifferentFields);
    }

    #[test]
    fn add_1() {
        let prime: u64 = 859;
        let num1: u64 = 17;
        let num2: u64 = 2222223; // 849 mod 859

        let a = U64FieldElement::new(num1, prime).unwrap();
        let b = U64FieldElement::new(num2, prime).unwrap();

        let res = a.add(&b).unwrap();
        assert_eq!(res, U64FieldElement { prime, num: 7 });
    }

    #[test]
    fn sub_err_different_primes() {
        let a = U64FieldElement::new(17, 797).unwrap();
        let b = U64FieldElement::new(17, 859).unwrap();

        let err = a.add(&b).unwrap_err();
        assert_eq!(err, Error::DifferentFields);
    }

    #[test]
    fn sub_1() {
        let prime: u64 = 859;
        let num1: u64 = 17;
        let num2: u64 = 2222223; // 849 mod 859

        let a = U64FieldElement::new(num1, prime).unwrap();
        let b = U64FieldElement::new(num2, prime).unwrap();

        let res = a.sub(&b).unwrap();
        assert_eq!(res, U64FieldElement { prime, num: 27 });
    }

    #[test]
    fn mul_1() {
        let prime: u64 = 859;
        let num1: u64 = 17;
        let num2: u64 = 2222223; // 849 mod 859

        let expected_result: u64 = 689;

        let a = U64FieldElement::new(num1, prime).unwrap();
        let b = U64FieldElement::new(num2, prime).unwrap();

        let res = a.mul(&b).unwrap();
        assert_eq!(
            res,
            U64FieldElement {
                prime,
                num: expected_result
            }
        );
    }

    #[test]
    fn exp_1() {
        let prime = 97;
        let a = U64FieldElement::new(3, prime).unwrap();
        let res = a.exp(&4).unwrap();
        assert_eq!(res, U64FieldElement { prime, num: 81 });
    }

    #[test]
    fn exp_2() {
        let prime = 97;
        let a = U64FieldElement::new(1, prime).unwrap();
        let res = a.exp(&326423784).unwrap();
        assert_eq!(res, U64FieldElement { prime, num: 1 });
    }

    #[test]
    fn exp_3() {
        let prime = 18446744073709551557;
        let a = U64FieldElement::new(2, prime).unwrap();
        let res = a.exp(&35).unwrap();
        assert_eq!(
            res,
            U64FieldElement {
                prime,
                num: 34359738368
            }
        );
    }

    #[test]
    fn test_div_ez() {
        let prime = 19;
        let a = U64FieldElement::new(2, prime).unwrap();
        let b = U64FieldElement::new(7, prime).unwrap();
        let c = U64FieldElement::new(3, prime).unwrap();

        assert_eq!(a.div(&b).unwrap(), c);
    }

    #[test]
    fn test_div_hard() {
        let prime = 19;
        let a = U64FieldElement::new(2, prime).unwrap();
        let b = U64FieldElement::new(7, prime).unwrap();
        let c = U64FieldElement::new(3, prime).unwrap();

        assert_eq!(a.div(&b).unwrap(), c);
    }
}

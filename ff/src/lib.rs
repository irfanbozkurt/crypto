pub mod errors;
use num_bigint::BigUint;

pub use crate::errors::{Error, Result};

pub mod utils;

#[derive(Debug, Clone)]
pub struct FieldElement {
    pub num: u64,
    pub prime: u64,
}

impl FieldElement {
    pub fn new(num: u64, prime: u64) -> Result<Self> {
        let is_prime = utils::miller_rabin(BigUint::from(prime));
        if !is_prime {
            return Err(Error::NotPrime);
        }

        let mut num = num;
        if num >= prime {
            num = ((num as u128) % (prime as u128)) as u64;
        }

        Ok(Self { num, prime })
    }

    pub fn add(&self, other: &Self) -> Result<Self> {
        Self::prime_check(&self.prime, &other.prime)?;

        let res = (((self.num as u128) + (other.num as u128)) % self.prime as u128) as u64;

        Ok(Self::new(res, self.prime)?)
    }

    pub fn sub(&self, other: &Self) -> Result<Self> {
        Self::prime_check(&self.prime, &other.prime)?;

        let res = (((self.num as u128 + self.prime as u128) - other.num as u128)
            % self.prime as u128) as u64;

        Ok(Self::new(res, self.prime)?)
    }

    pub fn mul(&self, other: &Self) -> Result<Self> {
        Self::prime_check(&self.prime, &other.prime)?;

        let res = (((self.num as u128) * (other.num as u128)) % self.prime as u128) as u64;

        Ok(Self::new(res, self.prime)?)
    }

    pub fn sq(&self) -> Result<Self> {
        self.mul(&self)
    }

    pub fn div(&self, other: &Self) -> Result<Self> {
        let other_inv = other.inv()?;
        let res = self.mul(&other_inv)?;
        Ok(res)
    }

    // Square & add algorithm
    pub fn exp(&self, exp: &u64) -> Result<Self> {
        // Use fermat's little theorem
        let mut exp = (*exp as u128 % (self.prime - 1) as u128) as u64;

        if exp == 0 {
            return Ok(Self {
                num: 0,
                prime: self.prime,
            });
        }

        let mut base = self.clone();
        let mut res = FieldElement {
            num: 1,
            prime: self.prime.clone(),
        };

        while exp != 0 {
            if exp & 1 == 1 {
                res = res.mul(&base)?;
            }
            base = base.sq()?;
            exp >>= 1;
        }

        return Ok(res);
    }

    // Uses Fermat's little
    pub fn inv(&self) -> Result<Self> {
        self.exp(&(&self.prime - 2))
    }

    fn prime_check(p_1: &u64, p_2: &u64) -> Result<()> {
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
        let err = FieldElement::new(17, 21).unwrap_err();
        assert_eq!(err, Error::NotPrime);
    }

    #[test]
    fn new_1() {
        let prime = 23;
        let num = 7871238;
        let a = FieldElement::new(num, prime).unwrap();
        assert_eq!(a, FieldElement { num: 17, prime });
    }

    #[test]
    fn cmp_neq_1() {
        let a = FieldElement::new(17, 23).unwrap();
        let b = FieldElement::new(16, 23).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn cmp_neq_2() {
        let c = FieldElement::new(17, 23).unwrap();
        let d = FieldElement::new(17, 29).unwrap();
        assert_ne!(c, d);
    }

    #[test]
    fn eq_1() {
        let a = FieldElement::new(17, 23).unwrap();
        let b = FieldElement::new(17, 23).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn add_err_different_primes() {
        let a = FieldElement::new(17, 797).unwrap();
        let b = FieldElement::new(17, 859).unwrap();

        let err = a.add(&b).unwrap_err();
        assert_eq!(err, Error::DifferentFields);
    }

    #[test]
    fn add_1() {
        let prime: u64 = 859;
        let num1: u64 = 17;
        let num2: u64 = 2222223; // 849 mod 859

        let a = FieldElement::new(num1, prime).unwrap();
        let b = FieldElement::new(num2, prime).unwrap();

        let res = a.add(&b).unwrap();
        assert_eq!(res, FieldElement { prime, num: 7 });
    }

    #[test]
    fn sub_err_different_primes() {
        let a = FieldElement::new(17, 797).unwrap();
        let b = FieldElement::new(17, 859).unwrap();

        let err = a.add(&b).unwrap_err();
        assert_eq!(err, Error::DifferentFields);
    }

    #[test]
    fn sub_1() {
        let prime: u64 = 859;
        let num1: u64 = 17;
        let num2: u64 = 2222223; // 849 mod 859

        let a = FieldElement::new(num1, prime).unwrap();
        let b = FieldElement::new(num2, prime).unwrap();

        let res = a.sub(&b).unwrap();
        assert_eq!(res, FieldElement { prime, num: 27 });
    }

    #[test]
    fn mul_1() {
        let prime: u64 = 859;
        let num1: u64 = 17;
        let num2: u64 = 2222223; // 849 mod 859

        let expected_result: u64 = 689;

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
        let prime = 97;
        let a = FieldElement::new(3, prime).unwrap();
        let res = a.exp(&4).unwrap();
        assert_eq!(res, FieldElement { prime, num: 81 });
    }

    #[test]
    fn exp_2() {
        let prime = 97;
        let a = FieldElement::new(1, prime).unwrap();
        let res = a.exp(&326423784).unwrap();
        assert_eq!(res, FieldElement { prime, num: 1 });
    }

    #[test]
    fn exp_3() {
        let prime = 18446744073709551557;
        let a = FieldElement::new(2, prime).unwrap();
        let res = a.exp(&35).unwrap();
        assert_eq!(
            res,
            FieldElement {
                prime,
                num: 34359738368
            }
        );
    }

    #[test]
    fn test_div_ez() {
        let prime = 19;
        let a = FieldElement::new(2, prime).unwrap();
        let b = FieldElement::new(7, prime).unwrap();
        let c = FieldElement::new(3, prime).unwrap();

        assert_eq!(a.div(&b).unwrap(), c);
    }

    #[test]
    fn test_div_hard() {
        let prime = 19;
        let a = FieldElement::new(2, prime).unwrap();
        let b = FieldElement::new(7, prime).unwrap();
        let c = FieldElement::new(3, prime).unwrap();

        assert_eq!(a.div(&b).unwrap(), c);
    }
}

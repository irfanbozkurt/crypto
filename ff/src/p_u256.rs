use std::str::FromStr;
use core::ops::{Add, Div, Mul, Neg, Sub};
use num_bigint::BigUint;
use primitive_types::U256;

use crate::errors::{Error, Result};
use utils::primality_test;

#[derive(Debug, Clone)]
pub struct U256FieldElement {
    pub num: U256,
    pub prime: U256,
}

/////////////////////////////////////////////
/////////////// Operator Overloads
/////////////////////////////////////////////
///// Equality
impl Eq for U256FieldElement {}
impl PartialEq for U256FieldElement {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num && self.prime == other.prime
    }
}
///// Addition
impl Add<&U256FieldElement> for &U256FieldElement {
    type Output = U256FieldElement;

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
    fn add(self, rhs: &Self::Output) -> Self::Output {
        Self::Output::is_same_field(&self.prime, &rhs.prime).unwrap();

        let (mut res, overflow) = self.num.overflowing_add(rhs.num);

        if overflow {
            res += (U256::MAX - self.prime) + 1;
        }

        res %= self.prime;

        Self::Output {
            num: res,
            prime: self.prime,
        }
    }
}
impl Add<U256FieldElement> for U256FieldElement {
    type Output = U256FieldElement;
    fn add(self, rhs: Self::Output) -> Self::Output {
        &self + &rhs
    }
}
impl Add<U256FieldElement> for &U256FieldElement {
    type Output = U256FieldElement;
    fn add(self, rhs: Self::Output) -> Self::Output {
        self + &rhs
    }
}
impl Add<&U256FieldElement> for U256FieldElement {
    type Output = U256FieldElement;
    fn add(self, rhs: &Self::Output) -> Self::Output {
        &self + rhs
    }
}
///// Subtraction
impl Sub<&U256FieldElement> for &U256FieldElement {
    type Output = U256FieldElement;

    /// (a(modp))-(b(modp)) (modp)  ==>  amodp + (-b)modp = amodp + (p-b)modp
    ///
    /// (a + p - b) is subject to overflows, but our addition function is already
    /// precautious against such situations
    fn sub(self, rhs: &Self::Output) -> Self::Output {
        Self::Output::is_same_field(&self.prime, &rhs.prime).unwrap();
        self + (-rhs)
    }
}
impl Sub<U256FieldElement> for U256FieldElement {
    type Output = U256FieldElement;
    fn sub(self, rhs: Self::Output) -> Self::Output {
        &self - &rhs
    }
}
impl Sub<U256FieldElement> for &U256FieldElement {
    type Output = U256FieldElement;
    fn sub(self, rhs: Self::Output) -> Self::Output {
        self - &rhs
    }
}
impl Sub<&U256FieldElement> for U256FieldElement {
    type Output = U256FieldElement;
    fn sub(self, rhs: &Self::Output) -> Self::Output {
        &self - rhs
    }
}
///// Multiplication
impl Mul<&U256FieldElement> for &U256FieldElement {
    type Output = U256FieldElement;

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
    fn mul(self, rhs: &Self::Output) -> Self::Output {
        Self::Output::is_same_field(&self.prime, &rhs.prime).unwrap();

        let mut rhs = rhs.num;
        if rhs == U256::zero() {
            return Self::Output {
                num: U256::zero(),
                prime: self.prime,
            };
        }

        let mut base = self.clone();
        let mut res = Self::Output {
            num: U256::zero(),
            prime: self.prime,
        };

        while rhs != U256::zero() {
            if rhs & U256::one() == U256::one() {
                res = res + &base;
            }
            base = (&base).double();
            rhs >>= 1;
        }

        res
    }
}
impl Mul<U256FieldElement> for U256FieldElement {
    type Output = U256FieldElement;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        &self * &rhs
    }
}
impl Mul<U256FieldElement> for &U256FieldElement {
    type Output = U256FieldElement;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        self * &rhs
    }
}
impl Mul<&U256FieldElement> for U256FieldElement {
    type Output = U256FieldElement;
    fn mul(self, rhs: &Self::Output) -> Self::Output {
        &self * rhs
    }
}
///// Division
impl Div<&U256FieldElement> for &U256FieldElement {
    type Output = U256FieldElement;
    fn div(self, rhs: &Self::Output) -> Self::Output {
        self * rhs.inv()
    }
}
impl Div<U256FieldElement> for U256FieldElement {
    type Output = U256FieldElement;
    fn div(self, rhs: Self::Output) -> Self::Output {
        &self / &rhs
    }
}
impl Div<&U256FieldElement> for U256FieldElement {
    type Output = U256FieldElement;
    fn div(self, rhs: &Self::Output) -> Self::Output {
        &self / rhs
    }
}
impl Div<U256FieldElement> for &U256FieldElement {
    type Output = U256FieldElement;
    fn div(self, rhs: Self::Output) -> Self::Output {
        self / &rhs
    }
}
///// Neg
impl Neg for &U256FieldElement {
    type Output = U256FieldElement;
    fn neg(self) -> Self::Output {
        Self::Output {
            num: self.prime - self.num,
            prime: self.prime,
        }
    }
}
impl Neg for U256FieldElement {
    type Output = U256FieldElement;
    fn neg(self) -> Self::Output {
        Self::Output {
            num: self.prime - self.num,
            prime: self.prime,
        }
    }
}


/////////////////////////////////////////////
/////////////// Field Requirements
/////////////////////////////////////////////
impl U256FieldElement {
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

    pub fn zero(prime: U256) -> Self {
        Self { num: U256::zero(), prime }
    }

    pub fn one(prime: U256) -> Self {
        Self { num: U256::one(), prime }
    }

    pub fn double(&self) -> Self {
        self + self
    }

    pub fn sq(&self) -> Self {
        self * self
    }

    // Uses Fermat's little
    pub fn inv(&self) -> Self {
        self.exp(&(&self.prime - 2))
    }

    pub fn exp_by_u64(&self, exp: u64) -> Self {
        self.exp(&U256::from(exp))
    }

    // Square & add algorithm
    pub fn exp(&self, exp: &U256) -> Self {
        // Use fermat's little theorem
        let mut exp = *exp % (self.prime - 1);

        if exp == U256::zero() {
            return Self {
                num: U256::zero(),
                prime: self.prime,
            };
        }

        let mut base = self.clone();
        let mut res = Self {
            num: U256::one(),
            prime: self.prime,
        };

        while exp != U256::zero() {
            if exp & U256::one() == U256::one() {
                res = res * &base;
            }
            base = base.sq();
            exp >>= 1;
        }

        return res;
    }

    fn is_same_field(p_1: &U256, p_2: &U256) -> Result<()> {
        if *p_1 != *p_2 {
            return Err(Error::DifferentFields);
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_err_not_a_prime() {
        let err = U256FieldElement::from_u64(17, 21).unwrap_err();
        assert_eq!(err, Error::NotPrime);
    }

    #[test]
    fn new_1() {
        let prime = U256::from(23);
        let num = U256::from(7871238);
        let a = U256FieldElement::new(num, prime).unwrap();
        assert_eq!(
            a,
            U256FieldElement {
                num: U256::from(17),
                prime
            }
        );
    }

    #[test]
    fn cmp_neq_1() {
        let a = U256FieldElement::from_u64(17, 23).unwrap();
        let b = U256FieldElement::from_u64(16, 23).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn cmp_neq_2() {
        let c = U256FieldElement::from_u64(17, 23).unwrap();
        let d = U256FieldElement::from_u64(17, 29).unwrap();
        assert_ne!(c, d);
    }

    #[test]
    fn eq_1() {
        let a = U256FieldElement::from_u64(17, 23).unwrap();
        let b = U256FieldElement::from_u64(17, 23).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn add_0() {
        let p = "0xB";

        let a = U256FieldElement::from_str("0xBD", &p).unwrap();
        let b = U256FieldElement::from_str("0x2B", &p).unwrap();

        let r = a + b;

        assert_eq!(
            r,
            U256FieldElement::from_str(
                "0000000000000000000000000000000000000000000000000000000000000001",
                &p
            )
            .unwrap()
        );
    }

    #[test]
    fn add_1() {
        let p = "0xf9cd";

        let a = U256FieldElement::from_str("0xa167f055ff75c", &p).unwrap();
        let b = U256FieldElement::from_str("0xacc457752e4ed", &p).unwrap();

        let r = a + b;

        assert_eq!(
            r,
            U256FieldElement::from_str(
                "0000000000000000000000000000000000000000000000000000000000006bb0",
                &p
            )
            .unwrap()
        );
    }

    #[test]
    fn add_2() {
        let p = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F";

        let a = U256FieldElement::from_str(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2E",
            &p,
        )
        .unwrap();
        let b = U256FieldElement::from_str(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2E",
            &p,
        )
        .unwrap();

        let r = a + b;

        assert_eq!(
            r,
            U256FieldElement::from_str(
                "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d",
                &p
            )
            .unwrap()
        );
    }

    // #[test]
    // fn add_3() {
    //     let a = U256FieldElement::from_u64(17, 797).unwrap();
    //     let b = U256FieldElement::from_u64(17, 859).unwrap();

    //     let err = a.add(&b).unwrap_err();
    //     assert_eq!(err, Error::DifferentFields);
    // }

    #[test]
    fn add_4() {
        let prime: U256 = U256::from(859);
        let num1: U256 = U256::from(17);
        let num2: U256 = U256::from(2222223); // 849 mod 859

        let a = U256FieldElement::new(num1, prime).unwrap();
        let b = U256FieldElement::new(num2, prime).unwrap();

        let res = a + b;
        assert_eq!(
            res,
            U256FieldElement {
                prime,
                num: U256::from(7)
            }
        );
    }

    // #[test]
    // fn sub_err_different_primes() {
    //     let a = U256FieldElement::from_u64(17, 797).unwrap();
    //     let b = U256FieldElement::from_u64(17, 859).unwrap();

    //     let err = a.add(&b).unwrap_err();
    //     assert_eq!(err, Error::DifferentFields);
    // }

    #[test]
    fn sub_1() {
        let prime: U256 = U256::from(859);
        let num1: U256 = U256::from(17);
        let num2: U256 = U256::from(2222223); // 849 mod 859

        let a = U256FieldElement::new(num1, prime).unwrap();
        let b = U256FieldElement::new(num2, prime).unwrap();

        let res = a - b;
        assert_eq!(
            res,
            U256FieldElement {
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

        let a = U256FieldElement::new(num1, prime).unwrap();
        let b = U256FieldElement::new(num2, prime).unwrap();

        let res = a * &b;
        assert_eq!(
            res,
            U256FieldElement {
                prime,
                num: expected_result
            }
        );
    }

    #[test]
    fn exp_1() {
        let prime = U256::from(97);
        let a = U256FieldElement::new(U256::from(3), prime).unwrap();
        let res = a.exp(&U256::from(4));
        assert_eq!(
            res,
            U256FieldElement {
                prime,
                num: U256::from(81)
            }
        );
    }

    #[test]
    fn exp_2() {
        let prime = U256::from(97);
        let a = U256FieldElement::new(U256::one(), prime).unwrap();
        let res = a.exp(&U256::from(326423784));
        assert_eq!(
            res,
            U256FieldElement {
                prime,
                num: U256::one()
            }
        );
    }

    #[test]
    fn exp_3() {
        let prime = U256::from_str("0xFFFFFFFFFFFFFFC5").unwrap();
        let a = U256FieldElement::new(U256::from(2), prime).unwrap();
        let res = a.exp(&U256::from(35));
        assert_eq!(
            res,
            U256FieldElement {
                prime,
                num: U256::from_str("0x800000000").unwrap()
            }
        );
    }

    #[test]
    fn test_div_ez() {
        let prime = 19;
        let a = U256FieldElement::from_u64(2, prime).unwrap();
        let b = U256FieldElement::from_u64(7, prime).unwrap();
        let c = U256FieldElement::from_u64(3, prime).unwrap();

        assert_eq!(a / b, c);
    }

    #[test]
    fn test_div_hard() {
        let prime = 19;
        let a = U256FieldElement::from_u64(2, prime).unwrap();
        let b = U256FieldElement::from_u64(7, prime).unwrap();
        let c = U256FieldElement::from_u64(3, prime).unwrap();

        assert_eq!(a / b, c);
    }
}

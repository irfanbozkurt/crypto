use crate::U256ECPoint;
use ff::p_u256::FieldElement;
use primitive_types::U256;
use std::str::FromStr;

pub struct Secp256k1;
// Constants
impl Secp256k1 {
    pub fn p_str() -> &'static str {
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F"
    }
    pub fn gx_str() -> &'static str {
        "0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798"
    }
    pub fn gy_str() -> &'static str {
        "0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8"
    }
    pub fn p() -> U256 {
        U256::from_str(Self::p_str()).unwrap()
    }
    pub fn g() -> U256ECPoint {
        U256ECPoint::from_str(Self::gx_str(), Self::gy_str(), Self::p_str())
    }
    pub fn identity() -> U256ECPoint {
        U256ECPoint::zero_zero(Self::p())
    }
}

/*
    Field operations will be defined within curve implementation
    and not within the point implementation as the arithmetics for
    chord and tangent rules are different for different curves and
    different representations
*/
impl Secp256k1 {
    pub fn add(p: &U256ECPoint, q: &U256ECPoint) -> U256ECPoint {
        if p.x == q.x {
            if p.y == q.y {
                return Self::double(p);
            }
            return Self::identity();
        }
        if p.is_identity() {
            return q.clone();
        }
        if q.is_identity() {
            return p.clone();
        }

        let slope = Self::calc_slope_chord(p, q);

        Self::add_by_slope(&slope, p, q)
    }

    pub fn double(p: &U256ECPoint) -> U256ECPoint {
        // Tangent doesn't intersect the curve, or is identity point
        if p.y.num.is_zero() {
            return Self::identity();
        }

        let slope = Self::calc_slope_tang(p);

        Self::add_by_slope(&slope, p, p)
    }

    /// Double & add algorithm
    pub fn exp(p: &U256ECPoint, exp: U256) -> U256ECPoint {
        if p.x.prime != Self::p() {
            panic!("Does not belong to this curve");
        }

        if exp.is_zero() {
            return p.clone();
        }

        let mut exp = exp;
        let mut base = p.clone();
        let mut res = Self::identity();

        while exp != U256::zero() {
            if exp & U256::one() == U256::one() {
                res = Secp256k1::add(&res, &base);
            }
            base = Secp256k1::double(&base);
            exp >>= 1;
        }

        res
    }

    /// dy / dx
    fn calc_slope_chord(p: &U256ECPoint, q: &U256ECPoint) -> FieldElement {
        let dx = p.x.sub(&q.x).expect("Field element subtraction failed");
        let dy = p.y.sub(&q.y).expect("Field element subtraction failed");
        dy.div(&dx).expect("Field element division failed")
    }

    /// s = ( 3 * x^2 + a) / 2 * y
    /// a is 0 in secp256k1, so it's just 3 * x^2  / 2 * y
    fn calc_slope_tang(p: &U256ECPoint) -> FieldElement {
        p.x.sq()
            .unwrap()
            .mul(&FieldElement::from_u64_and_u256_prime(3, p.x.prime).unwrap())
            .unwrap()
            .div(
                &p.y.mul(&FieldElement::from_u64_and_u256_prime(2, p.x.prime).unwrap())
                    .unwrap(),
            )
            .unwrap()
    }

    fn add_by_slope(slope: &FieldElement, p: &U256ECPoint, q: &U256ECPoint) -> U256ECPoint {
        let x3 = Self::calc_x_of_addition(&slope, &p.x, &q.x);
        let y3 = Self::calc_y_of_addition(&slope, &x3, &p.x, &p.y);
        U256ECPoint { x: x3, y: y3 }
    }

    /// 𝑥𝑟=𝜆2−𝑥𝑝−𝑥𝑞
    fn calc_x_of_addition(
        slope: &FieldElement,
        x1: &FieldElement,
        x2: &FieldElement,
    ) -> FieldElement {
        slope
            .sq()
            .expect("Squaring the slope failed")
            .sub(x1)
            .expect("Subtracting Px from the slope failed")
            .sub(x2)
            .expect("Subtracting Qx from the slope failed")
            .clone()
    }

    /// 𝑦𝑟=𝜆(𝑥𝑝−𝑥𝑟)−𝑦𝑝
    fn calc_y_of_addition(
        slope: &FieldElement,
        x3: &FieldElement,
        x1: &FieldElement,
        y1: &FieldElement,
    ) -> FieldElement {
        x1.sub(x3)
            .expect("Subtracting x3 from Px failed")
            .mul(slope)
            .expect("Multiplying by slope while finding y3 failed")
            .sub(y1)
            .expect("Subtracting Py while finding y3 failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_0() {
        let prime = Secp256k1::p();

        let p = Secp256k1::g();
        let q = U256ECPoint::zero_zero(prime);

        let r = Secp256k1::add(&p, &q);
        assert_eq!(r, p);

        let r = Secp256k1::add(&q, &p);
        assert_eq!(r, p);
    }

    #[test]
    fn add_1() {
        let prime = Secp256k1::p_str();

        let p = U256ECPoint::from_str(
            "0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
            prime,
        );
        let q = U256ECPoint::from_str(
            "0xC6047F9441ED7D6D3045406E95C07CD85C778E4B8CEF3CA7ABAC09B95C709EE5",
            "0x1AE168FEA63DC339A3C58419466CEAEEF7F632653266D0E1236431A950CFE52A",
            prime,
        );
        let r = Secp256k1::add(&p, &q);

        let expected_result = U256ECPoint::from_str(
            "0xf9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9",
            "0x388f7b0f632de8140fe337e62a37f3566500a99934c2231b6cb9fd7584b8e672",
            prime,
        );

        assert_eq!(r, expected_result);
    }

    #[test]
    fn double_0() {
        let prime = Secp256k1::p_str();

        let p = U256ECPoint::from_str(
            "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
            prime,
        );

        let p_square = Secp256k1::double(&p);
        let p_to_the_four = Secp256k1::double(&p_square);

        let expected_result = U256ECPoint::from_str(
            "e493dbf1c10d80f3581e4904930b1404cc6c13900ee0758474fa94abe8c4cd13",
            "51ed993ea0d455b75642e2098ea51448d967ae33bfbdfe40cfe97bdc47739922",
            prime,
        );

        assert_eq!(p_to_the_four, expected_result);
    }

    #[test]
    fn exp_0() {
        let prime = Secp256k1::p_str();

        let p = U256ECPoint::from_str(
            "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
            prime,
        );

        let p_square = Secp256k1::double(&p);
        let p_to_the_four = Secp256k1::double(&p_square);
        let p_to_the_eight = Secp256k1::double(&p_to_the_four);
        let p_to_the_sixteen = Secp256k1::double(&p_to_the_eight);
        let p_to_the_seventeen = Secp256k1::add(&p_to_the_sixteen, &p);

        let p_exp_seventeen = Secp256k1::exp(&p, U256::from(17));

        assert_eq!(p_to_the_seventeen, p_exp_seventeen);
    }
}

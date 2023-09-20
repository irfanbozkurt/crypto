use ff::p_u256::FieldElement;
use primitive_types::U256;

#[derive(Debug, Clone)]
pub struct U256ECPoint {
    pub x: FieldElement,
    pub y: FieldElement,
}

impl Eq for U256ECPoint {}
impl PartialEq for U256ECPoint {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl U256ECPoint {
    pub fn from_str(x: &str, y: &str, prime: &str) -> Self {
        Self {
            x: FieldElement::from_str(x, prime).unwrap(),
            y: FieldElement::from_str(y, prime).unwrap(),
        }
    }

    pub fn zero_zero(p: U256) -> Self {
        Self {
            x: FieldElement::new(U256::zero(), p).unwrap(),
            y: FieldElement::new(U256::zero(), p).unwrap(),
        }
    }

    pub fn is_identity(&self) -> bool {
        self.x.num.is_zero() && self.y.num.is_zero()
    }
}

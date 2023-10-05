use ff::p_u256::U256FieldElement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polynomial<FieldElement> {
	pub coefficients: Vec<FieldElement> // growing in degree
}

impl Polynomial<U256FieldElement> {
	pub fn new(coefficients: &[U256FieldElement]) -> Self {
		Self {
			coefficients: coefficients.iter().cloned().collect::<Vec<U256FieldElement>>()
		}
	}

	pub fn zero() -> Self {
		Self::new(&[])
	}

	pub fn degree(&self) -> usize {
		let mut degree = self.coefficients.len();
		if degree > 0 {degree -= 1;}
		degree
	}

	pub fn last_coefficient(&self) -> U256FieldElement {
		let coeff_len = self.coefficients.len();
		if coeff_len == 0 {
			U256FieldElement::from_str("0", "7").unwrap() // TODO: rearchitect the field
		} else {
			self.coefficients[coeff_len - 1].clone()
		}
	}

	pub fn batch_evaluate(&self, domain: &[U256FieldElement]) -> Vec<U256FieldElement> {
		domain.iter().map(|x| self.evaluate(x)).collect()
	}
	pub fn evaluate(&self, x: &U256FieldElement) -> U256FieldElement {
		self.horners_method(x)
	}
	fn horners_method(&self, x: &U256FieldElement) -> U256FieldElement {
		let mut val = U256FieldElement::zero(x.prime);
		for coeff in self.coefficients.iter().rev() {
			val = val * x + coeff;
		}
		val
	}

}

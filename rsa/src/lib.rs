pub mod errors;

pub use crate::errors::{Error, Result};

use num_bigint::{BigInt, BigUint, IntoBigInt, ModInverse, RandPrime, Sign::Plus};
use num_integer::Integer;
use num_traits::{FromPrimitive, One, Signed, Zero};

//////////////////////////////////////////////////////
//////////////////   Public Key  /////////////////////
//////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct PublicKey {
    n: BigUint,
}

impl PublicKey {
    pub const MAX_SIZE_MODULUS: usize = 4096;

    pub fn new(n: BigUint) -> Result<Self> {
        if n.bits() > PublicKey::MAX_SIZE_MODULUS {
            return Err(Error::ModulusTooLarge);
        }

        if n.is_even() {
            return Err(Error::EvenModulus);
        }

        Ok(Self { n })
    }

    pub fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>> {
        if msg.len() + 2 > self.size_bytes() {
            return Err(Error::MessageTooLong);
        }

        let e = BigUint::from_u64(65537).unwrap();

        Ok(BigUint::from_bytes_be(msg)
            .modpow(&e, &self.n)
            .to_bytes_be())
    }

    fn size_bytes(&self) -> usize {
        (self.n.bits() + 7) / 8
    }
}

//////////////////////////////////////////////////////
//////////////////   Private Key  ////////////////////
//////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct PrivateKey {
    public_key: PublicKey,
    p: BigUint,
    q: BigUint,
    /// Precomputed values to speed-up private ops
    /// d mod (p-1)
    d_mod_p_minus_one: BigUint,
    /// d mod (q-1)
    d_mod_q_minus_one: BigUint,
    /// q^{-1} mod (p)
    q_inv_mod_p_minus_one: BigInt,
}

struct PrivateKeyComponents {
    n: BigUint,
    d: BigUint,
    p: BigUint,
    q: BigUint,
}

impl PrivateKey {
    pub fn new(modulus_bit_size: usize) -> Result<PrivateKey> {
        let comps = PrivateKey::generate_private_key_components(modulus_bit_size)?;

        let d_mod_p_minus_one = &comps.d % (&comps.p - BigUint::one());
        let d_mod_q_minus_one = &comps.d % (&comps.q - BigUint::one());
        let q_inv_mod_p_minus_one = comps
            .p
            .clone()
            .mod_inverse(&comps.q)
            .ok_or(Error::InvalidPrime)?;

        Ok(Self {
            public_key: PublicKey::new(comps.n)?,
            p: comps.p,
            q: comps.q,
            d_mod_p_minus_one,
            d_mod_q_minus_one,
            q_inv_mod_p_minus_one,
        })
    }

    pub fn decrypt(&self, c: &[u8]) -> Result<Vec<u8>> {
        let c = BigUint::from_bytes_be(c);

        if c >= self.public_key.n || self.public_key.n.is_zero() {
            return Err(Error::Decryption);
        }

        let mut m1 = c
            .modpow(&self.d_mod_p_minus_one, &self.p)
            .into_bigint()
            .unwrap();
        let m2 = c
            .modpow(&self.d_mod_q_minus_one, &self.q)
            .into_bigint()
            .unwrap();

        m1 -= &m2;

        let p_bigint = BigInt::from_biguint(Plus, self.p.clone());
        while m1.is_negative() {
            m1 += &p_bigint;
        }

        m1 *= &self.q_inv_mod_p_minus_one;
        m1 %= p_bigint;

        let mut m = m1.to_biguint().unwrap();
        m *= &self.q;
        m += m2.to_biguint().unwrap();

        Ok(m.to_bytes_be())
    }

    pub fn get_public_key(&self) -> Result<PublicKey> {
        PublicKey::new(self.public_key.n.clone())
    }

    fn generate_private_key_components(modulus_bit_size: usize) -> Result<PrivateKeyComponents> {
        let mut p: BigUint;
        let mut q: BigUint;
        let mut n: BigUint;
        let d: BigUint;

        let e = BigUint::from_u64(65537).unwrap();
        let prime_bits = modulus_bit_size / 2;

        let mut rng = rand::thread_rng();

        // Generate the primes and reiterate if any check fails
        loop {
            p = rng.gen_prime(prime_bits);
            q = rng.gen_prime(prime_bits);

            if p == q {
                continue;
            }

            n = &p * &q;
            let totient = (&p - BigUint::one()) * (&q - BigUint::one());

            // `mod_inverse` returns `None` if gcd(e, totient) != 1
            // `e` must not be a factor of (p - 1) or (q - 1)
            if let Some(einv) = (&e).mod_inverse(totient) {
                d = einv.to_biguint().unwrap();
                break;
            }
        }

        Ok(PrivateKeyComponents { n, p, q, d })
    }
}

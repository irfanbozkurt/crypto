pub mod errors;

pub use crate::errors::{Error, Result};

use num_bigint::{BigInt, BigUint, ModInverse, RandPrime};
use num_integer::Integer;
use num_traits::{FromPrimitive, One, Zero};

//////////////////////////////////////////////////////
//////////////////   Public Key  /////////////////////
//////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct PublicKey {
    pub n: BigUint,
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
pub struct PrecomputedValues {
    /// d mod (p-1)
    pub d_mod_p_minus_one: BigUint,
    /// d mod (q-1)
    pub d_mod_q_minus_one: BigUint,
    /// q^{-1} mod (p)
    pub q_inv_mod_p_minus_one: BigInt,
}

#[derive(Debug, Clone)]
pub struct PrivateKey {
    public_key: PublicKey,
    pub d: BigUint,
    pub p: BigUint,
    pub q: BigUint,
    /// Precomputed values to speed-up private ops
    pub precomputed: PrecomputedValues,
}

pub struct PrivateKeyComponents {
    n: BigUint,
    d: BigUint,
    p: BigUint,
    q: BigUint,
}

impl PrivateKey {
    pub fn new(modulus_bit_size: usize) -> Result<PrivateKey> {
        let comps = PrivateKey::generate_private_key_components(modulus_bit_size)?;
        let precomputed = PrivateKey::precompute(&comps)?;

        Ok(Self {
            public_key: PublicKey::new(comps.n)?,
            d: comps.d,
            p: comps.p,
            q: comps.q,
            precomputed,
        })
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

    fn precompute(comps: &PrivateKeyComponents) -> Result<PrecomputedValues> {
        let d_mod_p_minus_one = &comps.d & (&comps.p - BigUint::one());
        let d_mod_q_minus_one = &comps.d & (&comps.q - BigUint::one());
        let q_inv_mod_p_minus_one = comps
            .p
            .clone()
            .mod_inverse(&comps.q)
            .ok_or(Error::InvalidPrime)?;

        Ok(PrecomputedValues {
            d_mod_p_minus_one,
            d_mod_q_minus_one,
            q_inv_mod_p_minus_one,
        })
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let c = BigUint::from_bytes_be(ciphertext);

        if c >= self.public_key.n || self.public_key.n.is_zero() {
            return Err(Error::Decryption);
        }

        Ok(c.modpow(&self.d, &self.public_key.n).to_bytes_be())
    }

    pub fn n(&self) -> &BigUint {
        &self.public_key.n
    }
}

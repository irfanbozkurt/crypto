use std::fmt::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Decryption,
    MessageTooLong,
    InvalidPrime,
    EvenModulus,
    ModulusTooLarge,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Decryption => write!(f, "decryption error"),
            Error::MessageTooLong => write!(f, "message too long"),
            Error::InvalidPrime => write!(f, "invalid prime value"),
            Error::EvenModulus => write!(f, "modulus cannot be even"),
            Error::ModulusTooLarge => write!(f, "modulus too small"),
        }
    }
}

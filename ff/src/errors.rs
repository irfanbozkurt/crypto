#[derive(Debug, PartialEq)]
pub enum Error {
    NotPrime,
    NotImplemented,
    DifferentFields,
}

pub type Result<T> = core::result::Result<T, Error>;

// Display trait
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotPrime => write!(f, "Provided prime is not a prime"),
            Error::DifferentFields => write!(f, "Elements belong to different prime fields"),
            Error::NotImplemented => write!(f, "Feature not implemented"),
        }
    }
}

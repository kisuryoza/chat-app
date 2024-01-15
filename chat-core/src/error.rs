pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Generic: {0}")]
    Generic(String),
    #[error("Cryptographic function failed: {0}")]
    Crypto(String),
    #[error("Decoding failed: {0}")]
    Decode(String),
    #[error("IO error: {0}")]
    IO(String),
    #[error("Timeout")]
    Timeout,
    #[error("Shutdown")]
    Shutdown,
}

unsafe impl Sync for Error {}
unsafe impl Send for Error {}

impl Error {
    pub fn generic<T: ToString>(msg: T) -> Self {
        Self::Generic(msg.to_string())
    }
    pub fn crypto<T: ToString>(msg: T) -> Self {
        Self::Crypto(msg.to_string())
    }
    pub fn decode<T: ToString>(msg: T) -> Self {
        Self::Decode(msg.to_string())
    }
    pub fn io<T: ToString>(msg: T) -> Self {
        Self::IO(msg.to_string())
    }
}

use std::{fmt, ops::Deref};

use crate::{crypto::Encodable, prelude::*};

pub const CRYPTO_KEY_LENGTH: usize = 32;
/// base64 url-safe encoded length
// pub const CRYPTO_KEY_LENGTH_ENCODED: usize = 43;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CryptoKey {
    bytes: [u8; CRYPTO_KEY_LENGTH],
}

pub type SecretKey = CryptoKey;
pub type PublicKey = CryptoKey;
pub type SharedSecret = CryptoKey;

impl CryptoKey {
    pub const fn new(bytes: [u8; CRYPTO_KEY_LENGTH]) -> Self {
        Self { bytes }
    }
}

impl Deref for CryptoKey {
    type Target = [u8; CRYPTO_KEY_LENGTH];
    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl AsRef<[u8]> for CryptoKey
where
    <Self as Deref>::Target: AsRef<[u8]>,
{
    fn as_ref(&self) -> &[u8] {
        &**self
    }
}

impl From<[u8; CRYPTO_KEY_LENGTH]> for CryptoKey {
    fn from(bytes: [u8; CRYPTO_KEY_LENGTH]) -> Self {
        Self { bytes }
    }
}

impl TryFrom<&[u8]> for CryptoKey {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();
        let bytes: [u8; CRYPTO_KEY_LENGTH] = value.try_into().map_err(|_| {
            Error::generic(format!(
                "Expected a slice of length {CRYPTO_KEY_LENGTH} but it was {len}"
            ))
        })?;
        Ok(Self { bytes })
    }
}

impl Encodable for CryptoKey {
    fn encode(&self) -> String {
        super::base64_encode(self)
    }

    fn try_decode<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let decoded = super::base64_decode(bytes)?;
        Self::try_from(decoded.as_slice())
    }
}

impl fmt::Display for CryptoKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct KeyPair {
    secret: SecretKey,
    public: PublicKey,
}

impl KeyPair {
    pub const fn new(secret: SecretKey, public: PublicKey) -> Self {
        Self { secret, public }
    }
    pub fn new_dh() -> Self {
        let secret = x25519_dalek::StaticSecret::random_from_rng(rand_core::OsRng);
        let public = x25519_dalek::PublicKey::from(&secret);
        Self {
            secret: secret.to_bytes().into(),
            public: public.to_bytes().into(),
        }
    }
    pub const fn into_split(self) -> (SecretKey, PublicKey) {
        (self.secret, self.public)
    }
    pub const fn secret(&self) -> &SecretKey {
        &self.secret
    }
    pub const fn public(&self) -> &PublicKey {
        &self.public
    }
}

/* impl Encodable for KeyPair {
    fn encode(&self) -> String {
        format!("{}:{}", self.secret.encode(), self.public.encode())
    }

    fn try_decode<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();
        if bytes.len() != CRYPTO_KEY_LENGTH_ENCODED * 2 + 1 {
            return Err(Error::generic(
                "Invalid encoded length for decoding from bytes to KeyPair",
            ));
        }
        let secret = SecretKey::try_decode(&bytes[0..CRYPTO_KEY_LENGTH_ENCODED])?;
        let public = PublicKey::try_decode(&bytes[(CRYPTO_KEY_LENGTH_ENCODED + 1)..])?;
        Ok(Self { secret, public })
    }
} */

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.secret.encode(), self.public.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_key() {
        let arr: [u8; 32] = rand::random();
        let vec = arr.to_vec();
        let key1 = CryptoKey::from(arr);
        let key2 = CryptoKey::try_from(vec.as_ref()).unwrap();
        assert_eq!(key1, key2);
        assert_eq!(*key1, *key2);
        assert_eq!(arr, *key1);
        assert_eq!(arr, *key2);

        let key_encoded = key1.encode();
        let key_decoded = CryptoKey::try_decode(key_encoded).unwrap();
        assert_eq!(key1, key_decoded);
    }

    /* #[test]
    fn key_pair() {
        let sec1: [u8; 32] = rand::random();
        let pub1: [u8; 32] = rand::random();
        let kp = KeyPair::new(sec1.into(), pub1.into());

        let kp_encoded = kp.encode();
        let kp_decoded = KeyPair::try_decode(kp_encoded.as_bytes()).unwrap();
        assert_eq!(kp, kp_decoded);

        let (sec2, pub2) = kp_decoded.into_split();
        assert_eq!(sec1, *sec2);
        assert_eq!(pub1, *pub2);
    } */
}

use crate::prelude::*;

mod _crypto;
mod types;

pub use _crypto::Crypto;
pub use types::*;

pub trait Encodable
where
    Self: Sized,
{
    fn encode(&self) -> String;
    fn try_decode<T: AsRef<[u8]>>(bytes: T) -> Result<Self>;
}

pub trait CryptoSchema {
    fn encrypt(&self, key: &SecretKey, blob: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, key: &SecretKey, blob: &[u8]) -> Result<Vec<u8>>;

    fn key_derivation(&self, pwd: &[u8], salt: &[u8]) -> Result<SecretKey>;
    fn hash_password(&self, plain_password: &[u8]) -> Result<String>;
    fn verify_password(&self, phc_string: &str, plain_password: &[u8]) -> Result<bool>;
    fn hash(&self, blob: &[u8]) -> [u8; 32];

    fn compute_dh(&self, secret: &SecretKey, public: &PublicKey) -> SharedSecret;
}

pub fn base64_encode<T: AsRef<[u8]>>(blob: T) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(blob.as_ref())
}

pub fn base64_decode<T: AsRef<[u8]>>(blob: T) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(blob.as_ref())
        .map_err(Error::decode)
}

const EMOJIES: [&str; 32] = [
    "🐸", "💖", "🐶", "🐳", "🐞", "🐢", "🐍", "🐔", "🏀", "🎹", "🍰", "🍪", "🍞", "🍒", "🍑", "🍎",
    "🍉", "🍄", "🍁", "🌻", "🌛", "🌑", "🌈", "⚡", "☕", "🚕", "🚀", "✅", "😍", "🚁", "🗿", "🔨",
];
pub fn key_to_emojies(shared_secret: &SharedSecret) -> String {
    let hashed = blake3::Hasher::new()
        .update(b"CORE_CRYPTO")
        .update(shared_secret.as_ref())
        .then(|f| f.finalize());

    let mut buf = String::with_capacity(32);
    let mut sum = 0;
    hashed.as_bytes().iter().enumerate().for_each(|(count, i)| {
        if count != 0 && count % 4 == 0 {
            buf.push_str(EMOJIES[sum % EMOJIES.len()]);
            sum = 0;
        }
        sum += *i as usize;
    });
    buf
}

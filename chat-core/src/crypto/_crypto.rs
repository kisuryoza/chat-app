use aead::{Aead, AeadCore, KeyInit};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand_core::OsRng;

use super::{CryptoSchema, Error, PublicKey, Result, SecretKey, SharedSecret, Then};

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
pub struct Crypto;

impl CryptoSchema for Crypto {
    fn encrypt(&self, key: &SecretKey, blob: &[u8]) -> Result<Vec<u8>> {
        let key = Key::from_slice(&**key);
        let cipher: XChaCha20Poly1305 = XChaCha20Poly1305::new(key);

        let nonce: XNonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 192-bits
        let mut ciphertext = cipher.encrypt(&nonce, blob).map_err(Error::crypto)?;

        let mut encrypted_text: Vec<u8> = Vec::with_capacity(nonce.len() + ciphertext.len());
        encrypted_text.append(&mut nonce.to_vec());
        encrypted_text.append(&mut ciphertext);

        Ok(encrypted_text)
    }
    fn decrypt(&self, key: &SecretKey, blob: &[u8]) -> Result<Vec<u8>> {
        let key = Key::from_slice(&**key);
        let cipher: XChaCha20Poly1305 = XChaCha20Poly1305::new(key);

        let (nonce, ciphertext) = blob.split_at(24);
        let nonce = XNonce::from_slice(nonce);
        let decrypted = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(Error::crypto)?;

        Ok(decrypted)
    }

    fn key_derivation(&self, pwd: &[u8], salt: &[u8]) -> Result<SecretKey> {
        if salt.len() != 32 {
            return Err(Error::crypto("invalid salt length"));
        }

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        let mut output_key_material = [0u8; 32];
        argon2
            .hash_password_into(pwd, salt, &mut output_key_material)
            .map_err(Error::crypto)?;

        Ok(SecretKey::new(output_key_material))
    }

    fn hash_password(&self, plain_password: &[u8]) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(plain_password, &salt)
            .map_err(Error::crypto)?
            .to_string();

        Ok(password_hash)
    }

    fn verify_password(&self, password_hash: &str, plain_password: &[u8]) -> Result<bool> {
        let parsed_hash = PasswordHash::new(password_hash).map_err(Error::crypto)?;

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        Ok(argon2.verify_password(plain_password, &parsed_hash).is_ok())
    }

    fn hash(&self, blob: &[u8]) -> [u8; 32] {
        let hash = blake3::hash(blob);
        *hash.as_bytes()
    }

    fn compute_dh(&self, secret: &SecretKey, public: &PublicKey) -> SharedSecret {
        let secret = x25519_dalek::StaticSecret::from(**secret);
        let public = x25519_dalek::PublicKey::from(**public);

        let shared_secret = secret.diffie_hellman(&public).to_bytes();

        let hashed = blake3::Hasher::new()
            .update(b"CORE_CRYPTO")
            .update(&shared_secret)
            .then(|f| f.finalize());

        SharedSecret::new(*hashed.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Crypto>();
    }
    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Crypto>();
    }
}

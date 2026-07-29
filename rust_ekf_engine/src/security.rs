use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use hkdf::Hkdf;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed (Invalid key, nonce, or tampered payload)")]
    DecryptionFailed,
    #[error("Key derivation failed")]
    KeyDerivationFailed,
    #[error("Base64 decoding failed")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Serialization failed")]
    SerializationError(#[from] serde_json::Error),
}

/// Enterprise Swarm Cryptography Module
/// Provides AES-256-GCM authenticated encryption for stigmergic threat data
/// and telemtry passed between the GCS and the Edge nodes.
pub struct CellhawkCrypto {
    cipher: Aes256Gcm,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedPayload {
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

impl CellhawkCrypto {
    /// Initializes the crypto module using a pre-shared master secret.
    /// In production, this master secret is injected via secure enclaves (e.g., TPM/AWS KMS)
    /// and derived using HKDF-SHA256 to generate the AES session key.
    pub fn new(master_secret: &[u8], salt: &[u8]) -> Result<Self, SecurityError> {
        let hk = Hkdf::<Sha256>::new(Some(salt), master_secret);
        let mut okm = [0u8; 32];
        hk.expand(b"cellhawk-swarm-key-v1", &mut okm)
            .map_err(|_| SecurityError::KeyDerivationFailed)?;

        let key = Key::<Aes256Gcm>::from_slice(&okm);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    /// Encrypts any serializable Rust struct into a secure base64-encoded envelope.
    pub fn encrypt_struct<T: Serialize>(
        &self,
        payload: &T,
    ) -> Result<EncryptedPayload, SecurityError> {
        let json_bytes = serde_json::to_vec(payload)?;

        // Generate a cryptographically secure random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, json_bytes.as_ref())
            .map_err(|_| SecurityError::EncryptionFailed)?;

        Ok(EncryptedPayload {
            nonce_b64: b64.encode(nonce_bytes),
            ciphertext_b64: b64.encode(ciphertext),
        })
    }

    /// Decrypts a base64-encoded envelope back into a strongly typed Rust struct.
    pub fn decrypt_struct<T: serde::de::DeserializeOwned>(
        &self,
        payload: &EncryptedPayload,
    ) -> Result<T, SecurityError> {
        let nonce_bytes = b64.decode(&payload.nonce_b64)?;
        let ciphertext = b64.decode(&payload.ciphertext_b64)?;

        if nonce_bytes.len() != 12 {
            return Err(SecurityError::DecryptionFailed);
        }

        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| SecurityError::DecryptionFailed)?;

        let struct_data: T = serde_json::from_slice(&plaintext)?;
        Ok(struct_data)
    }
}

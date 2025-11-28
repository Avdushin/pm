use crate::config::{Config, KdfParams, EncConfig};
use argon2::{Argon2, Algorithm, Params, Version};
use rand::RngCore;
use chacha20poly1305::{
    aead::{Aead, AeadCore, OsRng},
    KeyInit,
    XChaCha20Poly1305,
    XNonce,
};
use base64::{engine::general_purpose, Engine as _};
use thiserror::Error;
use anyhow::anyhow;

pub type MasterKey = [u8; 32];

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid master password")]
    InvalidMasterPassword,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Генерируем новый master key, шифруем его KEK'ом из мастер-пароля
/// и возвращаем готовый Config (kdf + enc).
pub fn generate_new_config(master_password: &str) -> anyhow::Result<Config> {
    // 1. Генерируем случайный MasterKey (MK)
    let mut mk = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut mk);

    // 2. KDF параметры (пока жёстко, потом можно сделать авто-бенчмарк)
    let mut salt_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let salt_b64 = general_purpose::STANDARD.encode(&salt_bytes);

    let kdf = KdfParams {
        algo: "argon2id".to_string(),
        memory_mib: 32,
        iterations: 3,
        parallelism: 1,
        salt: salt_b64,
    };

    // 3. Производим KEK из мастер-пароля
    let kek = derive_kek(master_password, &kdf)?;

    // 4. Шифруем MK KEK'ом (XChaCha20-Poly1305)
    let (nonce_b64, ct_b64) = encrypt_with_key(&kek, &mk)?;

    let enc = EncConfig {
        algo: "xchacha20-poly1305".to_string(),
        master_key_nonce: nonce_b64,
        encrypted_master_key: ct_b64,
    };

    Ok(Config {
        version: 1,
        kdf,
        enc,
    })
}

/// Расшифровка master key из config по мастер-паролю.
pub fn unlock_master_key(master_password: &str, cfg: &Config) -> Result<MasterKey, CryptoError> {
    let kek = derive_kek(master_password, &cfg.kdf)?;
    let mk = decrypt_with_key(
        &kek,
        &cfg.enc.master_key_nonce,
        &cfg.enc.encrypted_master_key,
    )?;
    Ok(mk)
}

/// Деривация KEK из мастер-пароля и KDF-параметров (Argon2id).
fn derive_kek(master_password: &str, kdf: &KdfParams) -> anyhow::Result<[u8; 32]> {
    let salt_bytes = general_purpose::STANDARD.decode(&kdf.salt)?;

    let params = Params::new(
        kdf.memory_mib * 1024,  // m_cost в KiB
        kdf.iterations,
        kdf.parallelism,
        Some(32),               // длина ключа
    ).map_err(|e| anyhow!("argon2 params error: {e}"))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut out = [0u8; 32];
    argon2
        .hash_password_into(
            master_password.as_bytes(),
            &salt_bytes,
            &mut out,
        )
        .map_err(|e| anyhow!("argon2 error: {e}"))?;

    Ok(out)
}

/// Шифрование произвольных данных с помощью заданного 32-байтного ключа.
/// Возвращает (nonce_b64, ciphertext_b64).
fn encrypt_with_key(key_bytes: &[u8; 32], plaintext: &[u8]) -> anyhow::Result<(String, String)> {
    let key = chacha20poly1305::Key::from_slice(key_bytes);
    let cipher = XChaCha20Poly1305::new(key);

    // XChaCha20 использует 24-байтный nonce
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow!("encrypt error: {e}"))?;

    let nonce_b64 = general_purpose::STANDARD.encode(&nonce);
    let ct_b64 = general_purpose::STANDARD.encode(&ciphertext);

    Ok((nonce_b64, ct_b64))
}

/// Дешифрование 32-байтного master key из nonce/ciphertext.
fn decrypt_with_key(key_bytes: &[u8; 32], nonce_b64: &str, ct_b64: &str) -> Result<MasterKey, CryptoError> {
    let key = chacha20poly1305::Key::from_slice(key_bytes);
    let cipher = XChaCha20Poly1305::new(key);

    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|e| CryptoError::Other(e.into()))?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = general_purpose::STANDARD
        .decode(ct_b64)
        .map_err(|e| CryptoError::Other(e.into()))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| CryptoError::InvalidMasterPassword)?;

    if plaintext.len() != 32 {
        return Err(CryptoError::Other(anyhow!("invalid master key length")));
    }

    let mut mk = [0u8; 32];
    mk.copy_from_slice(&plaintext);
    Ok(mk)
}

/// Шифрование JSON-записи master key'ом.
pub fn encrypt_entry(master_key: &MasterKey, data: &[u8]) -> anyhow::Result<(String, String)> {
    encrypt_with_key(master_key, data)
}

/// Дешифрование JSON-записи master key'ом.
pub fn decrypt_entry(master_key: &MasterKey, nonce_b64: &str, ct_b64: &str) -> anyhow::Result<Vec<u8>> {
    let key = chacha20poly1305::Key::from_slice(master_key);
    let cipher = XChaCha20Poly1305::new(key);

    let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64)?;
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = general_purpose::STANDARD.decode(ct_b64)?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow!("decrypt error: {e}"))?;
    Ok(plaintext)
}

/// Простая генерация пароля (позже можно сделать более кастомизируемой).
pub fn generate_password(len: usize, upper: bool, lower: bool, digits: bool) -> anyhow::Result<String> {
    let mut chars = String::new();
    if upper {
        chars.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if lower {
        chars.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if digits {
        chars.push_str("0123456789");
    }
    // базовый набор символов
    chars.push_str("!@#$%^&*()-_=+[]{};:,.<>?/");

    let chars: Vec<char> = chars.chars().collect();
    let mut rng = rand::thread_rng();
    let mut out = String::with_capacity(len);

    for _ in 0..len {
        let idx = (rng.next_u32() as usize) % chars.len();
        out.push(chars[idx]);
    }

    Ok(out)
}

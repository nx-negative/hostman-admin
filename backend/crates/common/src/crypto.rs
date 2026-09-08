//! Field encryption (AES-256-GCM), Crockford base32, argon2, Ed25519 root key, tokens.

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD as B64, URL_SAFE_NO_PAD as B64URL},
};
use ed25519_dalek::{Signer, SigningKey};
use rand::RngExt;
use sha2::{Digest, Sha256};
use std::path::Path;

pub type FieldKey = Key<Aes256Gcm>;

pub fn gen_field_key_b64() -> String {
    let mut k = [0u8; 32];
    rand::rng().fill(&mut k);
    B64.encode(k)
}

pub fn field_key_from_b64(s: &str) -> anyhow::Result<FieldKey> {
    let raw = B64.decode(s)?;
    anyhow::ensure!(raw.len() == 32, "FIELD_ENC_KEY must decode to 32 bytes");
    FieldKey::try_from(raw.as_slice()).map_err(|_| anyhow::anyhow!("bad key"))
}

/// AES-256-GCM; output = base64(nonce || ct).
pub fn encrypt_field(key: &FieldKey, pt: &str) -> anyhow::Result<String> {
    let cipher = Aes256Gcm::new(key);
    let mut n = [0u8; 12];
    rand::rng().fill(&mut n);
    let nonce = Nonce::try_from(n.as_slice()).map_err(|_| anyhow::anyhow!("nonce"))?;
    let ct = cipher.encrypt(&nonce, pt.as_bytes())?;
    Ok(B64.encode([n.as_slice(), ct.as_slice()].concat()))
}

pub fn decrypt_field(key: &FieldKey, b64: &str) -> anyhow::Result<String> {
    let raw = B64.decode(b64)?;
    anyhow::ensure!(raw.len() > 12, "ciphertext too short");
    let (n, ct) = raw.split_at(12);
    let nonce = Nonce::try_from(n).map_err(|_| anyhow::anyhow!("nonce"))?;
    let pt = Aes256Gcm::new(key).decrypt(&nonce, ct)?;
    Ok(String::from_utf8(pt)?)
}

pub fn hash_secret(s: &str) -> anyhow::Result<String> {
    Ok(Argon2::default().hash_password(s.as_bytes())?.to_string())
}

pub fn verify_secret(s: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|p| Argon2::default().verify_password(s.as_bytes(), &p).is_ok())
        .unwrap_or(false)
}

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub fn gen_login_code() -> String {
    let mut rng = rand::rng();
    (0..32)
        .map(|_| CROCKFORD[rng.random_range(0..32)] as char)
        .collect()
}

/// Uppercase, strip separators, map I/L→1 O→0 (Crockford).
pub fn normalize_code(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .map(|c| match c {
            'I' | 'L' => '1',
            'O' => '0',
            x => x,
        })
        .collect()
}

pub fn valid_code(s: &str) -> bool {
    s.len() == 32 && s.bytes().all(|b| CROCKFORD.contains(&b))
}

pub fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

pub fn gen_token() -> String {
    let mut b = [0u8; 48];
    rand::rng().fill(&mut b);
    B64URL.encode(b)
}

pub fn gen_recovery_code() -> String {
    let mut rng = rand::rng();
    let a: String = (0..5)
        .map(|_| CROCKFORD[rng.random_range(0..32)] as char)
        .collect();
    let b: String = (0..5)
        .map(|_| CROCKFORD[rng.random_range(0..32)] as char)
        .collect();
    format!("{a}-{b}")
}

/// Crockford-style base32 (I/L→1, O→0).
pub fn base32_encode(data: &[u8]) -> String {
    let mut acc: u64 = 0;
    let mut bits = 0u32;
    let mut out = String::new();
    for &b in data {
        acc = (acc << 8) | u64::from(b);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(CROCKFORD[((acc >> bits) & 0x1f) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(CROCKFORD[((acc << (5 - bits)) & 0x1f) as usize] as char);
    }
    out
}

pub fn base32_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    let mut acc: u64 = 0;
    let mut bits = 0u32;
    let mut out = Vec::new();
    for b in s.to_ascii_uppercase().bytes() {
        let b = match b {
            b'I' | b'L' => b'1',
            b'O' => b'0',
            b'=' | b'-' | b' ' => continue,
            x => x,
        };
        let v = CROCKFORD
            .iter()
            .position(|&c| c == b)
            .ok_or_else(|| anyhow::anyhow!("invalid base32 char"))? as u64;
        acc = (acc << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

/// Loads or creates the Ed25519 root key (raw 32-byte seed) at `path`.
pub fn ensure_root_key(path: &str) -> anyhow::Result<SigningKey> {
    if Path::new(path).exists() {
        let seed = std::fs::read(path)?;
        let arr: [u8; 32] = seed
            .try_into()
            .map_err(|_| anyhow::anyhow!("root key must be 32 bytes"))?;
        Ok(SigningKey::from_bytes(&arr))
    } else {
        let mut seed = [0u8; 32];
        rand::rng().fill(&mut seed);
        let key = SigningKey::from_bytes(&seed);
        if let Some(dir) = Path::new(path).parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, seed)?;
        Ok(key)
    }
}

/// EdDSA encoding/decoding keys for admin session JWTs, derived from the root key.
pub fn jwt_keys(
    sk: &SigningKey,
) -> anyhow::Result<(jsonwebtoken::EncodingKey, jsonwebtoken::DecodingKey)> {
    use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
    let pem = sk.to_pkcs8_pem(Default::default())?;
    let vpem = sk.verifying_key().to_public_key_pem(Default::default())?;
    Ok((
        jsonwebtoken::EncodingKey::from_ed_pem(pem.as_bytes())?,
        jsonwebtoken::DecodingKey::from_ed_pem(vpem.as_bytes())?,
    ))
}

pub fn sign(key: &SigningKey, msg: &[u8]) -> String {
    B64URL.encode(key.sign(msg).to_bytes())
}

pub fn verify_sig(vk: &ed25519_dalek::VerifyingKey, msg: &[u8], sig_b64: &str) -> bool {
    use ed25519_dalek::Verifier;
    let Ok(b) = B64URL.decode(sig_b64) else {
        return false;
    };
    let Ok(bytes) = <[u8; 64]>::try_from(b.as_slice()) else {
        return false;
    };
    vk.verify(msg, &ed25519_dalek::Signature::from_bytes(&bytes))
        .is_ok()
}

/// TOTP: verify `code` against base32 `secret` (±1 step per Appendix C).
pub fn verify_totp(secret_b32: &str, code: &str) -> bool {
    let Ok(bytes) = base32_decode(secret_b32) else {
        return false;
    };
    let Ok(t) = totp_rs::Builder::new()
        .with_algorithm(totp_rs::Algorithm::SHA1)
        .with_digits(6)
        .with_skew(1)
        .with_step_duration(30)
        .with_secret(bytes)
        .build()
    else {
        return false;
    };
    t.check_current(code).is_some()
}

/// Random TOTP secret as base32.
pub fn gen_totp_secret() -> String {
    let mut b = [0u8; 20];
    rand::rng().fill(&mut b);
    base32_encode(&b)
}

/// otpauth:// URL for QR enrollment.
pub fn totp_url(secret_b32: &str, account: &str) -> anyhow::Result<String> {
    let bytes = base32_decode(secret_b32)?;
    let t = totp_rs::Builder::new()
        .with_algorithm(totp_rs::Algorithm::SHA1)
        .with_digits(6)
        .with_skew(1)
        .with_step_duration(30)
        .with_secret(bytes)
        .with_issuer(Some("Hostman Admin"))
        .with_account_name(account)
        .build()?;
    Ok(t.to_url()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_roundtrip() {
        let key = field_key_from_b64(&gen_field_key_b64()).unwrap();
        let ct = encrypt_field(&key, "s3cret-PII").unwrap();
        assert_ne!(ct, "s3cret-PII");
        assert_eq!(decrypt_field(&key, &ct).unwrap(), "s3cret-PII");
    }

    #[test]
    fn crockford_normalize() {
        assert_eq!(normalize_code("ab01-ilO2"), "AB011102");
        assert!(valid_code(&gen_login_code()));
        assert!(!valid_code("SHORT"));
    }

    #[test]
    fn argon2_roundtrip() {
        let h = hash_secret("pw-123").unwrap();
        assert!(verify_secret("pw-123", &h));
        assert!(!verify_secret("nope", &h));
    }

    #[test]
    fn base32_roundtrip() {
        let data = b"hello world 123";
        assert_eq!(base32_decode(&base32_encode(data)).unwrap(), data);
    }

    #[test]
    fn sig_roundtrip() {
        let mut seed = [7u8; 32];
        rand::rng().fill(&mut seed);
        let sk = SigningKey::from_bytes(&seed);
        let sig = sign(&sk, b"payload");
        assert!(verify_sig(&sk.verifying_key(), b"payload", &sig));
        assert!(!verify_sig(&sk.verifying_key(), b"tampered", &sig));
    }

    #[test]
    fn jwt_eddsa_roundtrip() {
        let mut seed = [9u8; 32];
        rand::rng().fill(&mut seed);
        let sk = SigningKey::from_bytes(&seed);
        let (enc, dec) = jwt_keys(&sk).unwrap();
        let claims = serde_json::json!({"sub": "abc", "exp": 9999999999usize});
        let t = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::EdDSA),
            &claims,
            &enc,
        )
        .unwrap();
        let out: serde_json::Value = jsonwebtoken::decode(
            &t,
            &dec,
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::EdDSA),
        )
        .unwrap()
        .claims;
        assert_eq!(out["sub"], "abc");
    }

    #[test]
    fn totp_roundtrip() {
        let secret = gen_totp_secret();
        assert!(!verify_totp(&secret, "000000"));
    }
}

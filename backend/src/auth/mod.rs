use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::Engine;
use rand::rngs::OsRng;
use rsa::pkcs8::EncodePublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

/// 生成 2048-bit RSA 密钥对，返回 (private_key, public_key_spki_base64)
pub fn generate_rsa_keypair() -> anyhow::Result<(RsaPrivateKey, String)> {
    let private_key = RsaPrivateKey::new(&mut OsRng, 2048)?;
    let public_key = RsaPublicKey::from(&private_key);
    let spki_der = public_key.to_public_key_der()?;
    let pub_b64 = base64::engine::general_purpose::STANDARD.encode(spki_der.as_bytes());
    Ok((private_key, pub_b64))
}

/// RSA PKCS1v15 解密：base64 密文 → 明文
pub fn rsa_decrypt(private_key: &RsaPrivateKey, ciphertext_b64: &str) -> anyhow::Result<String> {
    let cipher_bytes = base64::engine::general_purpose::STANDARD.decode(ciphertext_b64)?;
    let plaintext = private_key.decrypt(Pkcs1v15Encrypt, &cipher_bytes)?;
    Ok(String::from_utf8(plaintext)?)
}

/// 密码哈希
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
        .to_string();
    Ok(hash)
}

/// 验证密码
pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("解析密码哈希失败: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

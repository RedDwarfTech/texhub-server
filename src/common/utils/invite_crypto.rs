//! 邀请凭证的加密原语。
//!
//! 为什么放在 texhub-server 而不是 rust_wheel：texhub-server 通过 crates.io
//! 依赖 rust_wheel（0.1.17），改本地 rust_wheel 源码对这里不可见，必须先发版
//! 才能被引用。为了一个功能局部用到的原语去推动公共库发版不划算，故就地实现。
//!
//! 依赖只用 `ring`（已在 Cargo.toml 中），SHA-256 走 `ring::digest`，不额外
//! 引入 sha2 / base64 / data-encoding / hex。

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::digest;
use ring::rand::{SecureRandom, SystemRandom};

/// 邀请码明文的原始字节数。
///
/// 32 字节 = 256 位熵，base64url 之后 43 个字符。远高于暴力猜解可行范围。
pub const TOKEN_BYTES: usize = 32;

/// 用操作系统 CSPRNG 生成 `len` 字节随机数。
///
/// 刻意不用 `rand::thread_rng` 之类的伪随机源：这里产出的是可以直接当凭证用的
/// 材料，可预测就等于没有凭证。
pub fn secure_random_bytes(len: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; len];
    SystemRandom::new()
        .fill(&mut buf)
        .map_err(|_| "failed to read from system CSPRNG".to_owned())?;
    Ok(buf)
}

/// 字节数组转小写十六进制。
fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// 邀请码的检索键：SHA-256 十六进制。
pub fn sha256_hex(data: &str) -> String {
    to_hex(digest::digest(&digest::SHA256, data.as_bytes()).as_ref())
}

/// base64url 编码（无 padding），结果可直接放进 URL query 而无需再转义。
///
/// 刻意手写而不引 base64 依赖：输入只来自我们自己生成的随机字节，长度固定，
/// 不存在被外部输入撑爆或触发解码分支的可能。
pub fn base64_url_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    // (len + 2) / 3 是向上机整再乘 4，不依赖 div_ceil 以规避 MSRV 问题。
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((n >> 6) & 63) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(n & 63) as usize] as char);
        }
    }
    out
}

/// AES-256-GCM 加密，输出 `nonce || ciphertext || tag`。
///
/// `aad` 是附加认证数据：不参与加密，但参与完整性校验。把它绑到 project_id
/// 上，攻击者就无法把一条密文搬到另一个项目下复用。
pub fn aes_gcm_encrypt(key: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    let unbound = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|_| "invalid AES-256-GCM key length".to_owned())?;
    let sealing_key = LessSafeKey::new(unbound);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce_bytes)
        .map_err(|_| "failed to generate AEAD nonce".to_owned())?;

    // in_out 原地变成 ciphertext || tag
    let mut in_out = plaintext.to_vec();
    sealing_key
        .seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::from(aad),
            &mut in_out,
        )
        .map_err(|_| "AES-256-GCM encryption failed".to_owned())?;

    let mut blob = Vec::with_capacity(NONCE_LEN + in_out.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&in_out);
    Ok(blob)
}

/// AES-256-GCM 解密，输入为 `nonce || ciphertext || tag`。
///
/// 认证失败（被篡改、key 不匹配、aad 不一致）统一返回 Err，不区分具体原因，
/// 避免把"密文格式对但 key 错"这类信息泄漏出去。
pub fn aes_gcm_decrypt(key: &[u8], blob: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    if blob.len() < NONCE_LEN {
        return Err("ciphertext is too short".to_owned());
    }
    let unbound = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|_| "invalid AES-256-GCM key length".to_owned())?;
    let opening_key = LessSafeKey::new(unbound);

    let nonce = Nonce::try_assume_unique_for_key(&blob[..NONCE_LEN])
        .map_err(|_| "invalid AEAD nonce".to_owned())?;
    let mut in_out = blob[NONCE_LEN..].to_vec();
    let plain = opening_key
        .open_in_place(nonce, Aad::from(aad), &mut in_out)
        .map_err(|_| "AES-256-GCM authentication failed".to_owned())?;
    Ok(plain.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Vec<u8> {
        // 32 字节测试密钥，非真实密钥，仅用于单测。
        (0u8..32).collect()
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = test_key();
        let cipher = aes_gcm_encrypt(&key, b"hello-token", b"proj-1").expect("encrypt");
        let plain = aes_gcm_decrypt(&key, &cipher, b"proj-1").expect("decrypt");
        assert_eq!(plain, b"hello-token");
    }

    #[test]
    fn decrypt_fails_with_wrong_aad() {
        // AAD 绑 project_id：把 proj-1 的密文拿到 proj-2 下解必须失败，
        // 否则邀请码就能跨项目搬运。
        let key = test_key();
        let cipher = aes_gcm_encrypt(&key, b"hello-token", b"proj-1").expect("encrypt");
        assert!(aes_gcm_decrypt(&key, &cipher, b"proj-2").is_err());
    }

    #[test]
    fn decrypt_fails_with_wrong_key() {
        let cipher = aes_gcm_encrypt(&test_key(), b"hello-token", b"proj-1").expect("encrypt");
        let other_key: Vec<u8> = (1u8..33).collect();
        assert!(aes_gcm_decrypt(&other_key, &cipher, b"proj-1").is_err());
    }

    #[test]
    fn decrypt_rejects_tampered_ciphertext() {
        // 改一个密文字节，GCM 认证必须失败 —— 这保证 token 不可篡改。
        let key = test_key();
        let mut cipher = aes_gcm_encrypt(&key, b"hello-token", b"proj-1").expect("encrypt");
        let last = cipher.len() - 1;
        cipher[last] ^= 0x01;
        assert!(aes_gcm_decrypt(&key, &cipher, b"proj-1").is_err());
    }

    #[test]
    fn decrypt_rejects_short_input() {
        assert!(aes_gcm_decrypt(&test_key(), &[0u8; 4], b"proj-1").is_err());
    }

    #[test]
    fn base64url_is_url_safe_and_unpadded() {
        let out = base64_url_encode(&secure_random_bytes(TOKEN_BYTES).expect("random"));
        // 无 '+' '/' '='，可直接进 query
        assert!(!out.contains('+') && !out.contains('/') && !out.contains('='));
        // 32 字节 -> 43 字符
        assert_eq!(out.len(), 43);
    }

    #[test]
    fn random_bytes_are_not_constant() {
        let a = secure_random_bytes(TOKEN_BYTES).expect("random");
        let b = secure_random_bytes(TOKEN_BYTES).expect("random");
        assert_ne!(a, b, "两次随机不应相同");
    }

    #[test]
    fn sha256_hex_is_stable() {
        // 固定输入 -> 固定输出，作为 token_hash 检索键必须可复现。
        assert_eq!(sha256_hex("abc"), sha256_hex("abc"));
        assert_ne!(sha256_hex("abc"), sha256_hex("abd"));
    }
}

use anyhow::Context;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn generate_timestamp_websocket() -> String {
    format!("{}", chrono::Utc::now().timestamp_millis())
}

/// OKX HMAC-SHA256 签名，返回 Base64
pub fn generate_signature(
    secret: &str,
    timestamp: &str,
    method: &reqwest::Method,
    path: &str,
    body: &str,
) -> anyhow::Result<String> {
    let payload = format!("{}{}{}{}", timestamp, method.as_str(), path, body);
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())?;
    mac.update(payload.as_bytes());
    let sig = mac.finalize().into_bytes();
    Ok(base64::engine::general_purpose::STANDARD.encode(sig))
}
pub fn sign_ed25519_base64(secret_or_pem: &str, payload: &str) -> anyhow::Result<String> {
    use ed25519_dalek::{Signer, SigningKey, pkcs8::DecodePrivateKey};

    // 1) 如果传入的是内联 PEM 内容，直接解析 PEM

    if secret_or_pem.contains("-----BEGIN") {
        let sk = ed25519_dalek::SigningKey::from_pkcs8_pem(secret_or_pem)
            .with_context(|| "解析 Ed25519 PKCS#8 PEM 私钥失败")?;

        let sig = sk.sign(payload.as_bytes());

        return Ok(base64::engine::general_purpose::STANDARD.encode(sig.to_bytes()));
    }

    // 2) 否则认为是文件路径，优先按文本读取并尝试 PEM；若非 PEM 再按二进制 DER 读取

    if let Ok(pem_text) = std::fs::read_to_string(secret_or_pem) {
        if pem_text.contains("-----BEGIN") {
            let sk = SigningKey::from_pkcs8_pem(&pem_text)
                .with_context(|| format!("解析 Ed25519 PEM 私钥失败: {}", secret_or_pem))?;

            let sig = sk.sign(payload.as_bytes());

            return Ok(base64::engine::general_purpose::STANDARD.encode(sig.to_bytes()));
        }
    }

    // 3) 按 DER 二进制尝试

    let der = std::fs::read(secret_or_pem)
        .with_context(|| format!("读取 Ed25519 私钥(DER)失败: {}", secret_or_pem))?;

    let sk =
        SigningKey::from_pkcs8_der(&der).with_context(|| "解析 Ed25519 PKCS#8 DER 私钥失败")?;

    let sig = sk.sign(payload.as_bytes());

    Ok(base64::engine::general_purpose::STANDARD.encode(sig.to_bytes()))
}

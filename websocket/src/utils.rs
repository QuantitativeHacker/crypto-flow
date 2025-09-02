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

/// Binance WS-API: 使用 Ed25519 对 payload 进行签名并返回 base64
/// secret_or_pem: 可以是内联 PEM 字符串，或指向 PEM 文件的路径
pub fn sign_ed25519_base64(secret_or_pem: &str, payload: &str) -> anyhow::Result<String> {
    use ed25519_dalek::{Signer, SigningKey, pkcs8::DecodePrivateKey};

    // 读取 PEM 内容
    let pem_str = if secret_or_pem.contains("-----BEGIN") {
        secret_or_pem.to_string()
    } else {
        std::fs::read_to_string(secret_or_pem)
            .with_context(|| format!("读取 Ed25519 私钥失败: {}", secret_or_pem))?
    };

    // 从 PEM 解析 Ed25519 SigningKey
    let sk = SigningKey::from_pkcs8_der(pem_str.as_bytes())
        .with_context(|| "解析 Ed25519 PKCS#8 私钥失败")?;

    let sig = sk.sign(payload.as_bytes());
    Ok(base64::engine::general_purpose::STANDARD.encode(sig.to_bytes()))
}

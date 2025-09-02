/// 统一凭证（当前 OKX 需要 passphrase；Binance 可忽略该字段）
#[derive(Clone, Debug)]
pub struct Credentials {
    pub api_key: String,
    /// 对 OKX：HMAC secret；对 Binance：可忽略或作为 REST 用密钥
    pub api_secret: String,
    /// 仅 OKX 需要；Binance 留空字符串
    pub passphrase: String,
    /// "0"(实盘) 或 "1"(模拟盘)，兼容现有调用
    pub is_simulated: String,
}

impl Credentials {
    pub fn new(
        api_key: String,
        api_secret: String,
        passphrase: String,
        is_simulated: &str,
    ) -> Self {
        Self {
            api_key,
            api_secret,
            passphrase,
            is_simulated: is_simulated.to_string(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct OkxWsAuth {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "sign")]
    pub sign: String,
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "passphrase")]
    pub passphrase: String,
}

#[derive(serde::Serialize)]
pub struct OkxWsLoginRequest {
    pub op: String,
    pub args: Vec<OkxWsAuth>,
}

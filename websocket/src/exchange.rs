use crate::auth::{Credentials, OkxWsAuth, OkxWsLoginRequest};
use crate::channel::{Args, ChannelType};
use crate::client::StoredSub;
use crate::request::{BinanceWsRequest, OkxSubscription, OkxWsOperation, OkxWsRequest};

/// 协议策略：定义各交易所 WS 消息格式
pub trait WsProtocol: Send + Sync {
    fn ping_text(&self) -> Option<String> {
        Some("ping".to_string())
    }

    fn build_login(&self, _cred: &Credentials) -> Option<serde_json::Value> {
        None
    }

    /// 构造订阅：返回可持久化的 StoredSub
    fn build_subscribe(&self, channel: ChannelType, args: &Args) -> StoredSub;

    /// 仅用于计算 HashMap 的 key，便于外部直接退订
    fn make_key(&self, channel: &ChannelType, args: &Args) -> String;
}

/// 提供各协议的默认端点
pub trait WsEndpoints {
    fn default_public_url() -> &'static str;
    fn default_private_url() -> Option<&'static str> {
        None
    }
}
/// OKX 协议实现
#[derive(Clone, Default)]
pub struct OkxProtocol;

impl WsProtocol for OkxProtocol {
    fn ping_text(&self) -> Option<String> {
        Some("ping".to_string())
    }

    fn build_login(&self, cred: &Credentials) -> Option<serde_json::Value> {
        let timestamp = crate::utils::generate_timestamp_websocket();
        let signature = crate::utils::generate_signature(
            &cred.api_secret,
            &timestamp,
            &reqwest::Method::GET,
            "/users/self/verify",
            "",
        )
        .ok()?;
        let auth = OkxWsAuth {
            api_key: cred.api_key.clone(),
            sign: signature,
            timestamp,
            passphrase: cred.passphrase.clone(),
        };
        let login_request = OkxWsLoginRequest {
            op: "login".to_string(),
            args: vec![auth],
        };
        serde_json::to_value(login_request).ok()
    }

    fn build_subscribe(&self, channel: ChannelType, args: &Args) -> StoredSub {
        let channel_name = Self::map_channel(&channel, args);
        let subscription = OkxSubscription {
            channel: channel_name.clone(),
            instrument_id: args.symbol().map(|s| s.to_string()),
            args: args.params.clone(),
        };
        let key = if let Some(ref inst_id) = subscription.instrument_id {
            format!("{}:{}", subscription.channel, inst_id)
        } else {
            subscription.channel.clone()
        };
        let req_sub = OkxWsRequest {
            op: OkxWsOperation::Subscribe,
            args: vec![subscription.clone()],
        };
        let req_unsub = OkxWsRequest {
            op: OkxWsOperation::Unsubscribe,
            args: vec![subscription.clone()],
        };
        StoredSub {
            key,
            local: Some(subscription),
            req_sub: serde_json::to_value(req_sub).unwrap(),
            req_unsub: serde_json::to_value(req_unsub).unwrap(),
        }
    }

    fn make_key(&self, channel: &ChannelType, args: &Args) -> String {
        let channel_name = Self::map_channel(channel, args);
        if let Some(inst) = &args.symbol() {
            format!("{}:{}", channel_name, inst)
        } else {
            channel_name
        }
    }
}

impl OkxProtocol {
    fn map_channel(channel: &ChannelType, _args: &Args) -> String {
        match channel {
            ChannelType::Candle(period) => format!("candle{}", period),
            ChannelType::Tickers => "tickers".to_string(),
            ChannelType::Trades => "trades".to_string(),
            ChannelType::Books => "books".to_string(),
            ChannelType::Depth => "depth".to_string(),
        }
    }
}

impl WsEndpoints for OkxProtocol {
    fn default_public_url() -> &'static str {
        "wss://ws.okx.com:8443/ws/v5/public"
    }
    fn default_private_url() -> Option<&'static str> {
        Some("wss://ws.okx.com:8443/ws/v5/private")
    }
}

/// Binance 协议实现（面向 stream 接口：支持 SUBSCRIBE/UNSUBSCRIBE）
#[derive(Clone, Default)]
pub struct BinanceProtocol;

impl BinanceProtocol {
    fn normalize_symbol(inst_id: &str) -> String {
        // 兼容 "BTC-USDT" 或 "btcusdt"
        inst_id.replace('-', "").to_lowercase()
    }

    fn map_channel(channel: &ChannelType, inst_id: &str, _args: &Args) -> String {
        let sym = Self::normalize_symbol(inst_id);
        match channel {
            ChannelType::Tickers => format!("{}@ticker", sym),
            ChannelType::Trades => format!("{}@trade", sym),
            ChannelType::Books => format!("{}@bookTicker", sym),
            ChannelType::Depth => {
                // 可根据 args.params 选择 depth 级别，默认标准 depth
                format!("{}@depth", sym)
            }
            ChannelType::Candle(period) => format!("{}@kline_{}", sym, period),
        }
    }
}

impl WsProtocol for BinanceProtocol {
    fn ping_text(&self) -> Option<String> {
        None
    }

    fn build_login(&self, _cred: &Credentials) -> Option<serde_json::Value> {
        // Binance 行情/stream 网关不支持会话登录（使用 SUBSCRIBE/UNSUBSCRIBE 即可），
        // 因此这里无需下发登录请求，直接返回 None。
        None
    }

    fn build_subscribe(&self, channel: ChannelType, args: &Args) -> StoredSub {
        let inst = args.normalized_symbol().unwrap_or_default();
        let param = Self::map_channel(&channel, &inst, args);
        let req_sub = BinanceWsRequest {
            method: "SUBSCRIBE".to_string(),
            params: vec![param.clone()],
            id: 1,
        };
        let req_unsub = BinanceWsRequest {
            method: "UNSUBSCRIBE".to_string(),
            params: vec![param],
            id: 1,
        };
        StoredSub {
            key: Self::map_channel(&channel, &inst, args),
            local: None,
            req_sub: serde_json::to_value(req_sub).unwrap(),
            req_unsub: serde_json::to_value(req_unsub).unwrap(),
        }
    }

    fn make_key(&self, channel: &ChannelType, args: &Args) -> String {
        let inst = args.normalized_symbol().unwrap_or_default();
        Self::map_channel(channel, &inst, args)
    }
}

impl WsEndpoints for BinanceProtocol {
    // 默认使用现货域名；用户可通过 set_url 改为合约域名
    fn default_public_url() -> &'static str {
        "wss://stream.binance.com:9443/ws"
    }
    // Binance stream 不区分私有端点，返回 None 表示与 public 相同（需自行拼 listenKey）
    // fn default_private_url() 使用默认实现即可
}

/// Binance WS-API（请求-响应）协议：支持 session.logon 会话鉴权
#[derive(Clone, Default)]
pub struct BinanceWsApiProtocol;

impl BinanceWsApiProtocol {
    /// 构造签名 payload：将参数按字母序排序后用 & 连接
    /// 参数格式：key=value&key2=value2...
    fn build_signature_payload(params: &std::collections::BTreeMap<String, String>) -> String {
        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// 为请求参数生成 Ed25519 签名
    fn sign_params(
        cred: &Credentials,
        params: &mut std::collections::BTreeMap<String, String>,
    ) -> Result<String, anyhow::Error> {
        // 构造签名 payload（不包含 signature 字段）
        let payload = Self::build_signature_payload(params);

        // 使用 Ed25519 对 payload 进行签名并返回 Base64 编码
        crate::utils::sign_ed25519_base64(&cred.api_secret, &payload)
    }

    /// 构建带签名的 WebSocket API 请求（如 order.place）
    /// 参数会自动按字母序排序并生成签名
    pub fn build_signed_request(
        &self,
        cred: &Credentials,
        id: &str,
        method: &str,
        mut params: std::collections::BTreeMap<String, serde_json::Value>,
    ) -> Option<serde_json::Value> {
        let timestamp = crate::utils::generate_timestamp_websocket();

        // 添加必需的认证参数
        params.insert(
            "apiKey".to_string(),
            serde_json::Value::String(cred.api_key.clone()),
        );
        params.insert(
            "timestamp".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                timestamp.parse::<i64>().unwrap_or(0),
            )),
        );

        // 将所有参数转换为字符串用于签名（除了 signature）
        let mut string_params = std::collections::BTreeMap::new();
        for (key, value) in &params {
            if key != "signature" {
                let string_value = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => value.to_string().trim_matches('"').to_string(),
                };
                string_params.insert(key.clone(), string_value);
            }
        }

        // 生成签名
        let signature = match Self::sign_params(cred, &mut string_params) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Ed25519 签名失败: {}", e);
                return None;
            }
        };

        // 添加签名到参数中
        params.insert(
            "signature".to_string(),
            serde_json::Value::String(signature),
        );

        // 构造最终请求
        let req = serde_json::json!({
            "id": id,
            "method": method,
            "params": params
        });

        tracing::info!(
            "构建 Binance WS-API 签名请求: method={}, apiKey={}",
            method,
            cred.api_key
        );
        Some(req)
    }
}

impl WsProtocol for BinanceWsApiProtocol {
    fn ping_text(&self) -> Option<String> {
        None
    }

    fn build_login(&self, cred: &Credentials) -> Option<serde_json::Value> {
        // 根据币安官方文档：https://developers.binance.com/docs/zh-CN/binance-spot-api-docs/websocket-api/authentication-requests
        // session.logon 需要 Ed25519 签名，仅支持 Ed25519 密钥
        let timestamp = crate::utils::generate_timestamp_websocket();

        // 构造参数映射（不包含 signature）
        let mut params = std::collections::BTreeMap::new();
        params.insert("apiKey".to_string(), cred.api_key.clone());
        params.insert("timestamp".to_string(), timestamp.clone());

        // 生成签名
        let signature = match Self::sign_params(cred, &mut params) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Ed25519 签名失败: {}", e);
                return None;
            }
        };

        // 构造 session.logon 请求
        let req = serde_json::json!({
            "id": "session_logon_1",
            "method": "session.logon",
            "params": {
                "apiKey": cred.api_key,
                "signature": signature,
                "timestamp": timestamp.parse::<i64>().unwrap_or(0)
            }
        });

        tracing::info!(
            "构建 Binance WS-API session.logon 请求: apiKey={}",
            cred.api_key
        );
        Some(req)
    }

    fn build_subscribe(&self, _channel: ChannelType, _args: &Args) -> StoredSub {
        // WS-API 不是订阅语义，这里返回占位，防止误用
        let req_sub = serde_json::json!({ "id": 1, "method": "session.status" });
        let req_unsub = serde_json::json!({ "id": 1, "method": "session.status" });
        StoredSub {
            key: "rpc".into(),
            local: None,
            req_sub,
            req_unsub,
        }
    }

    fn make_key(&self, _channel: &ChannelType, _args: &Args) -> String {
        "rpc".into()
    }
}

impl WsEndpoints for BinanceWsApiProtocol {
    fn default_public_url() -> &'static str {
        "wss://ws-api.binance.com/ws-api/v3"
    }
    fn default_private_url() -> Option<&'static str> {
        Some("wss://ws-api.binance.com/ws-api/v3")
    }
}

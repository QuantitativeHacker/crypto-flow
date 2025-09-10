use serde::{Deserialize, Serialize};
use serde_json::Value;

/// WebSocket API 通用响应结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsApiResponse<T> {
    /// 请求 ID，可以是字符串或数字
    pub id: Value,
    /// HTTP 状态码，200 表示成功
    pub status: u16,
    /// 响应结果，成功时包含具体数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    /// 错误信息，失败时包含错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<WsApiError>,
    /// 速率限制信息
    #[serde(rename = "rateLimits", skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<Vec<RateLimit>>,
}

/// WebSocket API 错误信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsApiError {
    /// 错误代码
    pub code: i32,
    /// 错误消息
    pub msg: String,
}

/// 速率限制信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimit {
    /// 速率限制类型
    #[serde(rename = "rateLimitType")]
    pub rate_limit_type: String,
    /// 时间间隔
    pub interval: String,
    /// 时间间隔数量
    #[serde(rename = "intervalNum")]
    pub interval_num: u32,
    /// 限制数量
    pub limit: u32,
    /// 当前计数
    pub count: u32,
}

/// session.logon 成功响应的结果部分
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionLogonResult {
    /// API Key
    #[serde(rename = "apiKey")]
    pub api_key: String,
    /// 认证开始时间（毫秒时间戳）
    #[serde(rename = "authorizedSince")]
    pub authorized_since: i64,
    /// 连接建立时间（毫秒时间戳）
    #[serde(rename = "connectedSince")]
    pub connected_since: i64,
    /// 是否返回速率限制信息
    #[serde(rename = "returnRateLimits")]
    pub return_rate_limits: bool,
    /// 服务器时间（毫秒时间戳）
    #[serde(rename = "serverTime")]
    pub server_time: i64,
    /// 用户数据流是否有效
    #[serde(rename = "userDataStream")]
    pub user_data_stream: bool,
}

/// session.logon 响应类型别名
pub type SessionLogonResponse = WsApiResponse<SessionLogonResult>;

/// session.status 响应的结果部分
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionStatusResult {
    /// 服务器时间（毫秒时间戳）
    #[serde(rename = "serverTime")]
    pub server_time: i64,
}

/// session.status 响应类型别名
pub type SessionStatusResponse = WsApiResponse<SessionStatusResult>;

/// session.logout 响应的结果部分
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionLogoutResult {
    /// 服务器时间（毫秒时间戳）
    #[serde(rename = "serverTime")]
    pub server_time: i64,
}

/// session.logout 响应类型别名
pub type SessionLogoutResponse = WsApiResponse<SessionLogoutResult>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_session_logon_response_deserialization() {
        let json = r#"{
            "id": "session_logon_1",
            "status": 200,
            "result": {
                "apiKey": "KZkPGaHZ036l59vQqoHV5ZEf2nEL2YKL8nCx300kSYPNGD1DuvitVvEqGIlBX8P3",
                "authorizedSince": 1757406257492,
                "connectedSince": 1757406257598,
                "returnRateLimits": true,
                "serverTime": 1757406257740,
                "userDataStream": false
            },
            "rateLimits": [
                {
                    "rateLimitType": "REQUEST_WEIGHT",
                    "interval": "MINUTE",
                    "intervalNum": 1,
                    "limit": 6000,
                    "count": 4
                }
            ]
        }"#;

        let response: SessionLogonResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "session_logon_1");
        assert_eq!(response.status, 200);
        assert!(response.result.is_some());
        
        let result = response.result.unwrap();
        assert_eq!(result.api_key, "KZkPGaHZ036l59vQqoHV5ZEf2nEL2YKL8nCx300kSYPNGD1DuvitVvEqGIlBX8P3");
        assert_eq!(result.authorized_since, 1757406257492);
        assert!(!result.user_data_stream);
    }

    #[test]
    fn test_session_error_response_deserialization() {
        let json = r#"{
            "id": "session_logon_1",
            "status": 400,
            "error": {
                "code": -1022,
                "msg": "Signature for this request is not valid."
            }
        }"#;

        let response: SessionLogonResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "session_logon_1");
        assert_eq!(response.status, 400);
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -1022);
        assert_eq!(error.msg, "Signature for this request is not valid.");
    }
}
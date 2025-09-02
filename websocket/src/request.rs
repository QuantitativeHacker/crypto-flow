use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct OkxWsRequest {
    pub op: OkxWsOperation,
    pub args: Vec<OkxSubscription>,
}

/// WebSocket订阅请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkxSubscription {
    /// 通道名称
    pub channel: String,
    /// 产品ID
    #[serde(rename = "instId", skip_serializing_if = "Option::is_none")]
    pub instrument_id: Option<String>,
    /// 额外参数
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub args: HashMap<String, String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OkxWsOperation {
    Subscribe,
    Unsubscribe,
}

// 可选：Binance 的简单请求结构（也可直接构建 JSON）
#[derive(serde::Serialize)]
pub struct BinanceWsRequest {
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
}

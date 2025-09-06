//! 订单簿深度订阅信息
//! see: https://developers.binance.com/docs/zh-CN/binance-spot-api-docs/websocket-api/market-data-requests#%E8%AE%A2%E5%8D%95%E8%96%84%E6%B7%B1%E5%BA%A6%E4%BF%A1%E6%81%AF

use serde::{Deserialize, Serialize};
use xcrypto::chat::GeneralDepth;

use crate::model::quote::BinanceQuote;

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceSpotDepth {
    pub stream: String,
    pub data: BinanceSpotDepthData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceSpotDepthData {
    pub bids: Vec<BinanceQuote>,
    pub asks: Vec<BinanceQuote>,
}

impl BinanceSpotDepth {
    pub fn stream(&self) -> &String {
        &self.stream
    }
}

impl From<BinanceSpotDepth> for GeneralDepth<BinanceQuote> {
    fn from(value: BinanceSpotDepth) -> Self {
        let time = now();
        let (symbol, stream) = value.stream.split_once("@").unwrap();

        match stream.split_once("@") {
            Some((_, interval)) => GeneralDepth {
                time,
                symbol: symbol.to_lowercase(),
                stream: format!("{}@depth:{}", symbol, interval).to_lowercase(),
                bids: value.data.bids,
                asks: value.data.asks,
            },
            None => GeneralDepth {
                time,
                symbol: symbol.to_lowercase(),
                stream: format!("{}@depth", symbol).to_lowercase(),
                bids: value.data.bids,
                asks: value.data.asks,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceFutureDepth {
    pub stream: String,
    pub data: BinanceFutureDepthData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceFutureDepthData {
    #[serde(rename = "E")]
    pub event_time: i64,
    pub s: String,
    pub b: Vec<BinanceQuote>,
    pub a: Vec<BinanceQuote>,
}

impl BinanceFutureDepth {
    pub fn stream(&self) -> &String {
        &self.stream
    }
}

impl From<BinanceFutureDepth> for GeneralDepth<BinanceQuote> {
    fn from(value: BinanceFutureDepth) -> Self {
        let (symbol, stream) = value.stream.split_once("@").unwrap();

        match stream.split_once("@") {
            Some((_, interval)) => GeneralDepth {
                time: value.data.event_time,
                symbol: symbol.to_lowercase(),
                stream: format!("{}@depth:{}", symbol, interval).to_lowercase(),
                bids: value.data.b,
                asks: value.data.a,
            },
            None => GeneralDepth {
                time: value.data.event_time,
                symbol: symbol.to_lowercase(),
                stream: format!("{}@depth", symbol).to_lowercase(),
                bids: value.data.b,
                asks: value.data.a,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_depth() {
        let s = r#"{"stream": "BTCUSDT@depth",
                          "data":{
                            "bids":[["64280.00000000","1.51596000"],
                                ["64279.81000000","0.18344000"],
                                ["64279.80000000","0.00000000"]],
                            "asks":[["64280.01000000","0.00000000"],
                                ["64280.02000000","0.00000000"],
                                ["64280.03000000","0.00000000"]]
                          }
                        }"#;
        let depth: BinanceSpotDepth = serde_json::from_str(s).unwrap();
        assert_eq!(depth.stream, "BTCUSDT@depth");
        assert_eq!(depth.data.bids.len(), 3);
        assert_eq!(depth.data.asks.len(), 3);
    }
}

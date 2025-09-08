//! see: https://developers.binance.com/docs/zh-CN/binance-spot-api-docs/testnet/web-socket-streams#klinecandlestick-streams-for-utc

use serde::{Deserialize, Serialize};
use cryptoflow::chat::GeneralKline;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceKline {
    pub stream: String,
    pub data: BinanceKlineData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BinanceKlineData {
    pub e: String,
    pub E: i64,
    pub s: String,
    pub k: BinanceKlineInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BinanceKlineInfo {
    pub t: i64,    // Kline start time
    pub T: i64,    // Kline close time
    pub s: String, // Symbol
    pub i: String, // Interval
    pub f: i64,    // First trade ID
    pub L: i64,    // Last trade ID
    pub o: String, // Open price
    pub c: String, // Close price
    pub h: String, // High price
    pub l: String, // Low price
    pub v: String, // Base asset volume
    pub n: i64,    // Number of trades
    pub x: bool,   // Is this kline closed?
    pub q: String, // Quote asset volume
    pub V: String, // Taker buy base asset volume
    pub Q: String, // Taker buy quote asset volume
    pub B: String, // Ignore
}

impl BinanceKline {
    pub fn stream(&self) -> &String {
        &self.stream
    }
}

impl From<BinanceKline> for GeneralKline {
    fn from(value: BinanceKline) -> Self {
        GeneralKline {
            time: value.data.k.T,       // K线结束时间
            start_time: value.data.k.t, // K线起始时间
            symbol: value.data.s.to_lowercase(),
            stream: format!("{}@kline:{}", value.data.s, value.data.k.i).to_lowercase(),
            interval: value.data.k.i,                           // K线间隔
            open: value.data.k.o.parse().unwrap_or_default(),   // 开盘价
            high: value.data.k.h.parse().unwrap_or_default(),   // 最高价
            low: value.data.k.l.parse().unwrap_or_default(),    // 最低价
            close: value.data.k.c.parse().unwrap_or_default(),  // 收盘价
            volume: value.data.k.v.parse().unwrap_or_default(), // 成交量
            amount: value.data.k.q.parse().unwrap_or_default(), // 成交额
            first_trade_id: value.data.k.f,                     // 第一笔成交ID
            last_trade_id: value.data.k.L,                      // 最后一笔成交ID
            trade_count: value.data.k.n,                        // 成交数量
            is_closed: value.data.k.x,                          // K线是否完结
            buy_volume: value.data.k.V.parse().unwrap_or_default(), // 主动买入成交量
            buy_amount: value.data.k.Q.parse().unwrap_or_default(), // 主动买入成交额
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_kline() {
        let s = r#"{
                            "stream": "bnbusdt@kline_1m",
                            "data":{
                                "e": "kline",     
                                "E": 1672515780000, 
                                "s": "BNBUSDT",    
                                "k": {
                                    "t": 1672515780000, 
                                    "T": 1672515839999, 
                                    "s": "BNBUSDT",     
                                    "i": "1m",         
                                    "f": 100,          
                                    "L": 200,          
                                    "o": "0.0010",     
                                    "c": "0.0020",     
                                    "h": "0.0025",     
                                    "l": "0.0015",     
                                    "v": "1000",       
                                    "n": 100,          
                                    "x": false,        
                                    "q": "1.0000",     
                                    "V": "500",        
                                    "Q": "0.500",      
                                    "B": "123456"      
                                }
                            }
                        }"#;
        let kline: BinanceKline = serde_json::from_str(s).unwrap();
        assert_eq!(kline.stream, "bnbusdt@kline_1m");
        assert_eq!(kline.data.e, "kline");
        assert_eq!(kline.data.E, 1672515780000);
        assert_eq!(kline.data.s, "BNBUSDT");
        assert_eq!(kline.data.k.t, 1672515780000);
        assert_eq!(kline.data.k.T, 1672515839999);
        assert_eq!(kline.data.k.s, "BNBUSDT");
        assert_eq!(kline.data.k.i, "1m");
        assert_eq!(kline.data.k.f, 100);
        assert_eq!(kline.data.k.L, 200);
        assert_eq!(kline.data.k.o, "0.0010");
        assert_eq!(kline.data.k.c, "0.0020");
        assert_eq!(kline.data.k.h, "0.0025");
        assert_eq!(kline.data.k.l, "0.0015");
        assert_eq!(kline.data.k.v, "1000");
        assert_eq!(kline.data.k.n, 100);
        assert_eq!(kline.data.k.x, false);
        assert_eq!(kline.data.k.q, "1.0000");
        assert_eq!(kline.data.k.V, "500");
        assert_eq!(kline.data.k.Q, "0.500");
        assert_eq!(kline.data.k.B, "123456");
    }
}

use serde::{Deserialize, Serialize};
use serde_json::Serializer;

use crate::model::{filter::FilterField, symbol::BinanceSymbol};

// 全局费率限制信息
#[derive(Debug, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct BinanceRateLimit {
    pub rateLimitType: String, // 限制类型: REQUEST_WEIGHT, ORDERS, CONNECTIONS
    pub interval: String,      // 限制间隔: SECOND, MINUTE, DAY
    pub intervalNum: u32,      // 间隔倍数
    pub limit: u32,            // 限制数量
    #[serde(default)]
    pub count: Option<u32>, // 当前计数（仅在响应中出现）
}

// 交易所信息（对应 exchangeInfo API 响应）
#[derive(Debug, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct BinanceExchangeInfo {
    pub timezone: String,                  // 时区
    pub serverTime: u64,                   // 服务器时间
    pub rateLimits: Vec<BinanceRateLimit>, // 全局费率限制
    #[serde(default)]
    pub exchangeFilters: Vec<serde_json::Value>, // 交易所过滤器
    pub symbols: Vec<BinanceSymbol>,       // 交易对列表
    #[serde(default)]
    pub sors: Option<Vec<serde_json::Value>>, // SOR 信息（可选）
}

#[cfg(test)]
mod tests {
    use crate::model::symbol::ConctactStatus;

    use super::*;

    #[test]
    fn test_binance_symbol() {
        // spot
        let s = r#"{"symbol": "ETHBTC", 
                          "status": "TRADING", 
                          "baseAsset": "ETH", 
                          "baseAssetPrecision": 8, 
                          "quoteAsset": "BTC", 
                          "quotePrecision": 8, 
                          "quoteAssetPrecision": 8, 
                          "baseCommissionPrecision": 8, 
                          "quoteCommissionPrecision": 8, 
                          "orderTypes": ["LIMIT", "LIMIT_MAKER", "MARKET", "STOP_LOSS_LIMIT", "TAKE_PROFIT_LIMIT"], 
                          "icebergAllowed": true, 
                          "ocoAllowed": true, 
                          "otoAllowed": false, 
                          "quoteOrderQtyMarketAllowed": true, 
                          "allowTrailingStop": true, 
                          "cancelReplaceAllowed": true, 
                          "isSpotTradingAllowed": true, 
                          "isMarginTradingAllowed": true, 
                          "filters": [{"filterType": "PRICE_FILTER", "minPrice": "0.00001000", "maxPrice": "922327.00000000", "tickSize": "0.00001000"}, 
                                      {"filterType": "LOT_SIZE", "minQty": "0.00010000", "maxQty": "100000.00000000", "stepSize": "0.00010000"}, 
                                      {"filterType": "ICEBERG_PARTS", "limit": 10}, 
                                      {"filterType": "MARKET_LOT_SIZE", "minQty": "0.00000000", "maxQty": "2703.20648368", "stepSize": "0.00000000"}, 
                                      {"filterType": "TRAILING_DELTA", "minTrailingAboveDelta": 10, "maxTrailingAboveDelta": 2000, "minTrailingBelowDelta": 10, "maxTrailingBelowDelta": 2000}, 
                                      {"filterType": "PERCENT_PRICE_BY_SIDE", "bidMultiplierUp": "5", "bidMultiplierDown": "0.2", "askMultiplierUp": "5", "askMultiplierDown": "0.2", "avgPriceMins": 5}, 
                                      {"filterType": "NOTIONAL", "minNotional": "0.00010000", "applyMinToMarket": true, "maxNotional": "9000000.00000000", "applyMaxToMarket": false, "avgPriceMins": 5}, 
                                      {"filterType": "MAX_NUM_ORDERS", "maxNumOrders": 200}, 
                                      {"filterType": "MAX_NUM_ALGO_ORDERS", "maxNumAlgoOrders": 5}], 
                            "permissions": [], 
                            "permissionSets": [["SPOT", "MARGIN", "TRD_GRP_004", "TRD_GRP_005", "TRD_GRP_006", "TRD_GRP_008", "TRD_GRP_009", "TRD_GRP_010", "TRD_GRP_011", "TRD_GRP_012", "TRD_GRP_013", "TRD_GRP_014", "TRD_GRP_015", "TRD_GRP_016", "TRD_GRP_017", "TRD_GRP_018", "TRD_GRP_019", "TRD_GRP_020", "TRD_GRP_021", "TRD_GRP_022", "TRD_GRP_023", "TRD_GRP_024", "TRD_GRP_025"]], 
                            "defaultSelfTradePreventionMode": "EXPIRE_MAKER", 
                            "allowedSelfTradePreventionModes": ["EXPIRE_TAKER", "EXPIRE_MAKER", "EXPIRE_BOTH"]}"#;
        let product: BinanceSymbol = serde_json::from_str(s).unwrap();
        assert_eq!(product.symbol, "ethbtc");
        assert_eq!(product.status, ConctactStatus::TRADING);
        assert_eq!(product.deliveryDate, None);
        assert_eq!(product.onboardDate, None);
        assert_eq!(product.filters.len(), 9);
        assert_eq!(product.orderTypes.len(), 5);

        // future
        let s = r#"{"symbol": "BTCUSDT", 
                          "pair": "BTCUSDT", 
                          "contractType": "PERPETUAL", 
                          "deliveryDate": 4133404800000, 
                          "onboardDate": 1569398400000, 
                          "status": "TRADING", 
                          "maintMarginPercent": "2.5000", 
                          "requiredMarginPercent": "5.0000", 
                          "baseAsset": "BTC", 
                          "quoteAsset": "USDT", 
                          "marginAsset": "USDT", 
                          "pricePrecision": 2, 
                          "quantityPrecision": 3, 
                          "baseAssetPrecision": 8, 
                          "quotePrecision": 8, 
                          "underlyingType": "COIN", 
                          "underlyingSubType": ["PoW"], 
                          "settlePlan": 0, 
                          "triggerProtect": "0.0500", 
                          "liquidationFee": "0.012500", 
                          "marketTakeBound": "0.05", 
                          "maxMoveOrderLimit": 10000, 
                          "filters": [{"maxPrice": "4529764", "minPrice": "556.80", "filterType": "PRICE_FILTER", "tickSize": "0.10"}, 
                                      {"stepSize": "0.001", "maxQty": "1000", "filterType": "LOT_SIZE", "minQty": "0.001"}, 
                                      {"minQty": "0.001", "stepSize": "0.001", "filterType": "MARKET_LOT_SIZE", "maxQty": "120"}, 
                                      {"limit": 200, "filterType": "MAX_NUM_ORDERS"}, 
                                      {"filterType": "MAX_NUM_ALGO_ORDERS", "limit": 10}, 
                                      {"notional": "100", "filterType": "MIN_NOTIONAL"}, 
                                      {"multiplierDecimal": "4", "filterType": "PERCENT_PRICE", "multiplierDown": "0.9500", "multiplierUp": "1.0500"}], 
                          "orderTypes": ["LIMIT", "MARKET", "STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET", "TRAILING_STOP_MARKET"], 
                          "timeInForce": ["GTC", "IOC", "FOK", "GTX", "GTD"]}"#;
        let product: BinanceSymbol = serde_json::from_str(s).unwrap();

        assert_eq!(product.symbol, "btcusdt");
        assert_eq!(product.status, ConctactStatus::TRADING);
        assert_eq!(product.deliveryDate, Some(4133404800000));
        assert_eq!(product.onboardDate, Some(1569398400000));
        assert_eq!(product.filters.len(), 7);
        assert_eq!(product.orderTypes.len(), 7);
    }

    #[test]
    fn test_binance_exchange_info() {
        let s = r#"{"timezone": "UTC", 
                          "serverTime": 1715054406944, 
                          "futuresType": "U_MARGINED", 
                          "rateLimits": [{"rateLimitType": "REQUEST_WEIGHT", "interval": "MINUTE", "intervalNum": 1, "limit": 2400}, 
                                         {"rateLimitType": "ORDERS", "interval": "MINUTE", "intervalNum": 1, "limit": 1200}, 
                                         {"rateLimitType": "ORDERS", "interval": "SECOND", "intervalNum": 10, "limit": 300}], 
                          "exchangeFilters": [], 
                          "assets": [{"asset": "USDT", "marginAvailable": true, "autoAssetExchange": "-10000"}, 
                                     {"asset": "BTC", "marginAvailable": true, "autoAssetExchange": "-0.10000000"}, 
                                     {"asset": "BNB", "marginAvailable": true, "autoAssetExchange": "-10"}, 
                                     {"asset": "ETH", "marginAvailable": true, "autoAssetExchange": "-5"}, 
                                     {"asset": "XRP", "marginAvailable": true, "autoAssetExchange": "0"},
                                     {"asset": "USDC", "marginAvailable": true, "autoAssetExchange": "-10000"}, 
                                     {"asset": "TUSD", "marginAvailable": true, "autoAssetExchange": "0"}, 
                                     {"asset": "FDUSD", "marginAvailable": true, "autoAssetExchange": "0"}], 
                          "symbols": [{"symbol": "BTCUSDT", 
                                       "pair": "BTCUSDT", 
                                       "contractType": "PERPETUAL", 
                                       "deliveryDate": 4133404800000, 
                                       "onboardDate": 1569398400000, 
                                       "status": "TRADING", 
                                       "maintMarginPercent": "2.5000", 
                                       "requiredMarginPercent": "5.0000", 
                                       "baseAsset": "BTC", 
                                       "quoteAsset": "USDT", 
                                       "marginAsset": "USDT", 
                                       "pricePrecision": 2, 
                                       "quantityPrecision": 3, 
                                       "baseAssetPrecision": 8, 
                                       "quotePrecision": 8, 
                                       "underlyingType": "COIN", 
                                       "underlyingSubType": ["PoW"], 
                                       "settlePlan": 0, 
                                       "triggerProtect": "0.0500", 
                                       "liquidationFee": "0.012500", 
                                       "marketTakeBound": "0.05", 
                                       "maxMoveOrderLimit": 10000, 
                                       "filters": [{"tickSize": "0.10", "maxPrice": "4529764", "filterType": "PRICE_FILTER", "minPrice": "556.80"}, 
                                                   {"minQty": "0.001", "stepSize": "0.001", "filterType": "LOT_SIZE", "maxQty": "1000"}, 
                                                   {"minQty": "0.001", "filterType": "MARKET_LOT_SIZE", "maxQty": "120", "stepSize": "0.001"}, 
                                                   {"filterType": "MAX_NUM_ORDERS", "limit": 200}, 
                                                   {"limit": 10, "filterType": "MAX_NUM_ALGO_ORDERS"}, 
                                                   {"notional": "100", "filterType": "MIN_NOTIONAL"}, 
                                                   {"multiplierDecimal": "4", "multiplierUp": "1.0500", "multiplierDown": "0.9500", "filterType": "PERCENT_PRICE"}], 
                                       "orderTypes": ["LIMIT", "MARKET", "STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET", "TRAILING_STOP_MARKET"], 
                                       "timeInForce": ["GTC", "IOC", "FOK", "GTX", "GTD"]}, 
                                      {"symbol": "ETHUSDT", 
                                       "pair": "ETHUSDT", 
                                       "contractType": "PERPETUAL", 
                                       "deliveryDate": 4133404800000, 
                                       "onboardDate": 1569398400000, 
                                       "status": "TRADING", 
                                       "maintMarginPercent": "2.5000", 
                                       "requiredMarginPercent": "5.0000", 
                                       "baseAsset": "ETH", 
                                       "quoteAsset": "USDT", 
                                       "marginAsset": "USDT", 
                                       "pricePrecision": 2, 
                                       "quantityPrecision": 3, 
                                       "baseAssetPrecision": 8, 
                                       "quotePrecision": 8, 
                                       "underlyingType": "COIN", 
                                       "underlyingSubType": ["Layer-1"], 
                                       "settlePlan": 0, 
                                       "triggerProtect": "0.0500", 
                                       "liquidationFee": "0.012500", 
                                       "marketTakeBound": "0.05", 
                                       "maxMoveOrderLimit": 10000, 
                                       "filters": [{"filterType": "PRICE_FILTER", "minPrice": "39.86", "tickSize": "0.01", "maxPrice": "306177"}, 
                                                   {"maxQty": "10000", "stepSize": "0.001", "filterType": "LOT_SIZE", "minQty": "0.001"}, 
                                                   {"filterType": "MARKET_LOT_SIZE", "minQty": "0.001", "stepSize": "0.001", "maxQty": "2000"}, 
                                                   {"filterType": "MAX_NUM_ORDERS", "limit": 200}, 
                                                   {"filterType": "MAX_NUM_ALGO_ORDERS", "limit": 10}, 
                                                   {"filterType": "MIN_NOTIONAL", "notional": "20"}, 
                                                   {"multiplierUp": "1.0500", "multiplierDown": "0.9500", "filterType": "PERCENT_PRICE", "multiplierDecimal": "4"}], 
                                       "orderTypes": ["LIMIT", "MARKET", "STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET", "TRAILING_STOP_MARKET"], 
                                       "timeInForce": ["GTC", "IOC", "FOK", "GTX", "GTD"]}]}"#;
        let rsp: BinanceExchangeInfo = serde_json::from_str(&s).unwrap();
        println!("{:?}", rsp);
    }
}

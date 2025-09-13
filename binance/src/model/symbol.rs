use cryptoflow::trading_rules::TradingRules;
use serde::{Deserialize, Serialize};

use super::deserialize_symbol;
use crate::model::filter::FilterField;

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ConctactStatus {
    TRADING,
    HALT,
    BREAK,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct BinanceSymbol {
    #[serde(deserialize_with = "deserialize_symbol")]
    pub symbol: String, // 交易对符号
    pub status: ConctactStatus,                       // 交易状态
    pub baseAsset: String,                            // 基础资产
    pub baseAssetPrecision: u8,                       // 基础资产精度
    pub quoteAsset: String,                           // 计价资产
    pub quotePrecision: u8,                           // 计价精度
    pub quoteAssetPrecision: u8,                      // 计价资产精度
    pub baseCommissionPrecision: u8,                  // 基础资产手续费精度
    pub quoteCommissionPrecision: u8,                 // 计价资产手续费精度
    pub orderTypes: Vec<String>,                      // 支持的订单类型
    pub icebergAllowed: bool,                         // 是否支持冰山订单
    pub ocoAllowed: bool,                             // 是否支持OCO订单
    pub otoAllowed: bool,                             // 是否支持OTO订单
    pub quoteOrderQtyMarketAllowed: bool,             // 是否支持市价单按计价资产数量下单
    pub allowTrailingStop: bool,                      // 是否支持跟踪止损
    pub cancelReplaceAllowed: bool,                   // 是否支持撤单改单
    pub amendAllowed: bool,                           // 是否支持修改订单
    pub pegInstructionsAllowed: bool,                 // 是否支持挂钩指令
    pub isSpotTradingAllowed: bool,                   // 是否支持现货交易
    pub isMarginTradingAllowed: bool,                 // 是否支持杠杆交易
    pub filters: Vec<FilterField>,                    // 交易规则过滤器
    pub permissions: Vec<String>,                     // 权限列表
    pub permissionSets: Vec<Vec<String>>,             // 权限集合
    pub defaultSelfTradePreventionMode: String,       // 默认自成交防护模式
    pub allowedSelfTradePreventionModes: Vec<String>, // 允许的自成交防护模式
    #[serde(default)]
    pub deliveryDate: Option<u64>, // 交割日期（期货合约）
    #[serde(default)]
    pub onboardDate: Option<u64>, // 上线日期
}

impl TradingRules for BinanceSymbol {
    fn symbol(&self) -> &String {
        &self.symbol
    }

    fn min_price(&self) -> f64 {
        for filter in &self.filters {
            if let FilterField::PRICE_FILTER { min_price, .. } = filter {
                return min_price.parse::<f64>().unwrap_or(0.0);
            }
        }
        0.0
    }

    fn max_price(&self) -> f64 {
        for filter in &self.filters {
            if let FilterField::PRICE_FILTER { max_price, .. } = filter {
                return max_price.parse::<f64>().unwrap_or(f64::MAX);
            }
        }
        f64::MAX
    }

    fn tick_size(&self) -> f64 {
        for filter in &self.filters {
            if let FilterField::PRICE_FILTER { tick_size, .. } = filter {
                return tick_size.parse::<f64>().unwrap_or(0.0);
            }
        }
        0.0
    }

    fn min_quantity(&self) -> f64 {
        for filter in &self.filters {
            if let FilterField::LOT_SIZE { min_qty, .. } = filter {
                return min_qty.parse::<f64>().unwrap_or(0.0);
            }
        }
        0.0
    }

    fn max_quantity(&self) -> f64 {
        for filter in &self.filters {
            if let FilterField::LOT_SIZE { max_qty, .. } = filter {
                return max_qty.parse::<f64>().unwrap_or(f64::MAX);
            }
        }
        f64::MAX
    }

    fn lot_size(&self) -> f64 {
        for filter in &self.filters {
            if let FilterField::LOT_SIZE { step_size, .. } = filter {
                return step_size.parse::<f64>().unwrap_or(0.0);
            }
        }
        0.0
    }

    fn min_notional(&self) -> f64 {
        for filter in &self.filters {
            match filter {
                FilterField::MIN_NOTIONAL { min_notional, .. } => {
                    return min_notional.parse::<f64>().unwrap_or(0.0);
                }
                FilterField::NOTIONAL { min_notional, .. } => {
                    return min_notional.parse::<f64>().unwrap_or(0.0);
                }
                _ => continue,
            }
        }
        0.0
    }
}

use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use super::deserialize_symbol;
use crate::model::filter::FilterField;

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum ConctactStatus {
    TRADING,
    HALT,
    BREAK,
}

#[derive(Debug, Deserialize, Clone)]
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

impl Serialize for BinanceSymbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Size {
            size: f64,
            max: f64,
            min: f64,
        }

        let mut state = serializer.serialize_struct("BinanceSymbol", 5)?;
        state.serialize_field("symbol", &self.symbol.to_lowercase())?;
        state.serialize_field("delivery", &self.deliveryDate)?;
        state.serialize_field("onboard", &self.onboardDate)?;
        state.serialize_field("order", &self.orderTypes)?;

        for filter in self.filters.iter() {
            match filter {
                FilterField::PRICE_FILTER {
                    tick_size,
                    max_price,
                    min_price,
                } => {
                    state.serialize_field(
                        "price_filter",
                        &Size {
                            size: tick_size.parse::<f64>().unwrap_or(0.0),
                            max: max_price.parse::<f64>().unwrap_or(0.0),
                            min: min_price.parse::<f64>().unwrap_or(0.0),
                        },
                    )?;
                }
                FilterField::LOT_SIZE {
                    step_size,
                    max_qty,
                    min_qty,
                } => {
                    state.serialize_field(
                        "lot_size",
                        &Size {
                            size: step_size.parse::<f64>().unwrap_or(0.0),
                            max: max_qty.parse::<f64>().unwrap_or(0.0),
                            min: min_qty.parse::<f64>().unwrap_or(0.0),
                        },
                    )?;
                }
                FilterField::NOTIONAL { min_notional, .. } => {
                    state.serialize_field(
                        "min_notional",
                        &min_notional.parse::<f64>().unwrap_or(0.0),
                    )?;
                }
                FilterField::MIN_NOTIONAL { min_notional, .. } => {
                    state.serialize_field(
                        "min_notional",
                        &min_notional.parse::<f64>().unwrap_or(0.0),
                    )?;
                }
                _ => (),
            }
        }
        state.end()
    }
}

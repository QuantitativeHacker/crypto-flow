use native_json::{is_default, json};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request<T> {
    pub id: i64,
    pub method: String,
    pub params: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<T> {
    pub id: i64,
    pub result: T,
}

json! {
    PositionReq {
        session_id: u16,
        symbols: Vec<String>,
    }
}

json! {
    PositionRsp {
        session_id: u16,
        positions: Vec<Position>,
    }
}

json! {
    Error {
    code: i32,
    msg: String,
    }
}

json! {
Login {
    session_id: u16,
    name: String?,
    trading: bool,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneralDepth<T> {
    pub time: i64,
    pub symbol: String,
    pub stream: String,
    pub bids: Vec<T>,
    pub asks: Vec<T>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneralKline {
    pub time: i64,       // 这根K线的结束时间 (T)
    pub start_time: i64, // 这根K线的起始时间 (t)
    pub symbol: String,
    pub stream: String,
    pub interval: String,    // K线间隔 (i)
    pub open: f64,           // 开盘价 (o)
    pub high: f64,           // 最高价 (h)
    pub low: f64,            // 最低价 (l)
    pub close: f64,          // 收盘价 (c)
    pub volume: f64,         // 成交量 (v)
    pub amount: f64,         // 成交额 (q)
    pub first_trade_id: i64, // 第一笔成交ID (f)
    pub last_trade_id: i64,  // 最后一笔成交ID (L)
    pub trade_count: i64,    // 成交数量 (n)
    pub is_closed: bool,     // 这根K线是否完结 (x)
    pub buy_volume: f64,     // 主动买入成交量 (V)
    pub buy_amount: f64,     // 主动买入成交额 (Q)
}

/// 订单信息
#[derive(Debug, Serialize)]
pub struct Order {
    pub internal_id: u32,
    pub state: State,
    pub order_id: i64,
    pub symbol: String,
    pub side: Side,
    pub type_: OrderType,
    pub tif: TimeInForce,
    pub price: f64,
    pub quantity: f64,

    pub trade_time: i64,
    pub trade_price: f64,
    pub trade_quantity: f64,
    pub acc: f64,
    pub making: bool,
}

impl Order {
    pub fn new(
        id: u32,
        symbol: String,
        side: Side,
        state: State,
        type_: OrderType,
        tif: TimeInForce,
        quantity: f64,
        price: f64,
    ) -> Self {
        Self {
            internal_id: id,
            symbol,
            side,
            state,
            type_,
            tif,
            quantity,
            price,
            order_id: -1,
            trade_time: 0,
            trade_price: 0.0,
            trade_quantity: 0.0,
            acc: 0.0,
            making: false,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum State {
    // Binance和OKX共有状态
    CANCELED,         // 用户撤销了订单
    PARTIALLY_FILLED, // 部分订单已被成交
    FILLED,           // 订单已完全成交

    // Binance 状态
    NEW,            // 该订单被交易引擎接受
    PENDING_NEW,    // 该订单处于待处理阶段，直到其所属订单组中的 working order 完全成交
    PENDING_CANCEL, // 撤销中(目前并未使用)
    REJECTED,       // 订单没有被交易引擎接受，也没被处理
    EXPIRED,        // 该订单根据订单类型的规则被取消或被交易引擎取消
    #[allow(non_camel_case_types)]
    EXPIRED_IN_MATCH, // 表示订单由于 STP 而过期

    // OKX 特有状态
    LIVE, // 等待成交 (OKX)
    #[allow(non_camel_case_types)]
    MMP_CANCELED, // 做市商保护机制导致的自动撤单 (OKX)
}

impl std::str::FromStr for State {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // Binance 状态
            "NEW" => Ok(State::NEW),
            "PENDING_NEW" => Ok(State::PENDING_NEW),
            "PARTIALLY_FILLED" => Ok(State::PARTIALLY_FILLED),
            "FILLED" => Ok(State::FILLED),
            "CANCELED" => Ok(State::CANCELED),
            "PENDING_CANCEL" => Ok(State::PENDING_CANCEL),
            "REJECTED" => Ok(State::REJECTED),
            "EXPIRED" => Ok(State::EXPIRED),
            "EXPIRED_IN_MATCH" => Ok(State::EXPIRED_IN_MATCH),

            // OKX 状态
            "live" => Ok(State::LIVE),
            "canceled" => Ok(State::CANCELED),
            "partially_filled" => Ok(State::PARTIALLY_FILLED),
            "filled" => Ok(State::FILLED),
            "mmp_canceled" => Ok(State::MMP_CANCELED),

            _ => Err(()),
        }
    }
}

impl State {
    /// 将状态转换为 Binance 格式的字符串
    pub fn to_binance_str(&self) -> &'static str {
        match self {
            State::NEW => "NEW",
            State::PENDING_NEW => "PENDING_NEW",
            State::PARTIALLY_FILLED => "PARTIALLY_FILLED",
            State::FILLED => "FILLED",
            State::CANCELED => "CANCELED",
            State::PENDING_CANCEL => "PENDING_CANCEL",
            State::REJECTED => "REJECTED",
            State::EXPIRED => "EXPIRED",
            State::EXPIRED_IN_MATCH => "EXPIRED_IN_MATCH",
            State::LIVE => "NEW", // OKX 的 live 对应 Binance 的 NEW
            State::MMP_CANCELED => "CANCELED", // OKX 的 mmp_canceled 对应 Binance 的 CANCELED
        }
    }

    /// 将状态转换为 OKX 格式的字符串
    pub fn to_okx_str(&self) -> &'static str {
        match self {
            State::NEW => "live",
            State::PENDING_NEW => "live",
            State::PARTIALLY_FILLED => "partially_filled",
            State::FILLED => "filled",
            State::CANCELED => "canceled",
            State::PENDING_CANCEL => "live",
            State::REJECTED => "canceled",
            State::EXPIRED => "canceled",
            State::EXPIRED_IN_MATCH => "canceled",
            State::LIVE => "live",
            State::MMP_CANCELED => "mmp_canceled",
        }
    }

    /// 从 Binance 状态字符串创建状态
    pub fn from_binance_str(s: &str) -> Result<Self, ()> {
        match s {
            "NEW" => Ok(State::NEW),
            "PENDING_NEW" => Ok(State::PENDING_NEW),
            "PARTIALLY_FILLED" => Ok(State::PARTIALLY_FILLED),
            "FILLED" => Ok(State::FILLED),
            "CANCELED" => Ok(State::CANCELED),
            "PENDING_CANCEL" => Ok(State::PENDING_CANCEL),
            "REJECTED" => Ok(State::REJECTED),
            "EXPIRED" => Ok(State::EXPIRED),
            "EXPIRED_IN_MATCH" => Ok(State::EXPIRED_IN_MATCH),
            _ => Err(()),
        }
    }

    /// 从 OKX 状态字符串创建状态
    pub fn from_okx_str(s: &str) -> Result<Self, ()> {
        match s {
            "live" => Ok(State::LIVE),
            "canceled" => Ok(State::CANCELED),
            "partially_filled" => Ok(State::PARTIALLY_FILLED),
            "filled" => Ok(State::FILLED),
            "mmp_canceled" => Ok(State::MMP_CANCELED),
            _ => Err(()),
        }
    }
}

/// Side of an order, Buy or Sell
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Side {
    BUY,
    SELL,
}

impl FromStr for Side {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BUY" => Ok(Self::BUY),
            "SELL" => Ok(Self::SELL),
            _ => unreachable!(),
        }
    }
}

/// Type of an order, Limit, Market, Stop, etc.
/// Reference:
/// https://developers.binance.com/docs/zh-CN/binance-spot-api-docs/testnet/websocket-api/trading-requests#place-new-order-trade
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[allow(non_camel_case_types)]
pub enum OrderType {
    LIMIT,
    MARKET,
    STOP_LOSS,
    STOP_LOSS_LIMIT,
    TAKE_PROFIT,
    TAKE_PROFIT_LIMIT,
    LIMIT_MAKER,
}

impl FromStr for OrderType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LIMIT" => Ok(Self::LIMIT),
            "MARKET" => Ok(Self::MARKET),
            "STOP_LOSS" => Ok(Self::STOP_LOSS),
            "STOP_LOSS_LIMIT" => Ok(Self::STOP_LOSS_LIMIT),
            "TAKE_PROFIT" => Ok(Self::TAKE_PROFIT),
            "TAKE_PROFIT_LIMIT" => Ok(Self::TAKE_PROFIT_LIMIT),
            "LIMIT_MAKER" => Ok(Self::LIMIT_MAKER),
            _ => unreachable!(),
        }
    }
}

/// Reference:
/// https://developers.binance.com/docs/zh-CN/binance-spot-api-docs/enums#%E7%94%9F%E6%95%88%E6%97%B6%E9%97%B4-timeinforce
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
}

impl FromStr for TimeInForce {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GTC" => Ok(Self::GTC),
            "IOC" => Ok(Self::IOC),
            "FOK" => Ok(Self::FOK),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Position {
    pub symbol: String,
    pub net: f64,
}

pub type Success = Response<Option<u8>>;
pub type LoginResponse = Response<Login>;
pub type ErrorResponse = Response<Error>;

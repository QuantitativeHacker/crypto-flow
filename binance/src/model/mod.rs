#![allow(non_snake_case)]
pub mod bookticker;
pub mod depth;
pub mod exchangeinfo;
pub mod filter;
pub mod kline;
pub mod order;
pub mod quote;
pub mod symbol;

use native_json::json;
use serde::{Deserialize, Deserializer, Serialize};
use xcrypto::chat::*;

use crate::{
    model::{
        bookticker::BinanceBookTicker,
        depth::{BinanceFutureDepth, BinanceSpotDepth},
        kline::BinanceKline,
        order::usdt::OrderUpdate,
    },
    OrderTrait,
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MarketStream {
    BookTicker(BinanceBookTicker),
    SpotDepth(BinanceSpotDepth),
    FutureDepth(BinanceFutureDepth),
    Kline(BinanceKline),
}

json! {
    SpotPosition {
        a: String,
        f: String,
        l: String
    }
}

json! {
    OutboundAccountPosition {
        e: String,
        E: i64,
        u: i64,
        B: [SpotPosition]
    }
}

json! {
    BalanceUpdate {
        e: String,
        E: i64,
        a: String,
        d: String,
        T: i64
    }
}

json! {
    SpotExpired {
        e: String,
        E: String,
        listenKey: String
    }
}

json! {
    UserLiabilityChange {
        e: String,
        E: i64,
        a: String,
        t: String,
        p: String,
        i: String,
    }
}

json! {
    MarginLevelStatusChange {
        e: String,
        E: i64,
        l: String,
        s: String
    }
}

json! {
    OCODetails{
        s: String,
        i: i64,
        c: String
    }
}

json! {
    ListenStatus {
        e: String,
        E: i64,
        s: String,
        g: i64,
        o: String,
        l: String,
        L: String,
        r: String,
        C: String,
        T: i64,
        O: [OCODetails]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(into = "Order")]
pub struct ExecutionReport {
    pub e: String, // 事件类型
    pub E: i64,    // 事件时间
    #[serde(deserialize_with = "deserialize_symbol")]
    pub s: String, // 交易对
    pub c: String, // clientOrderId
    pub S: Side,   // 订单方向
    pub o: String, // 订单类型
    pub f: String, // 有效方式
    pub q: String, // 订单原始数量
    pub p: String, // 订单原始价格
    pub P: String, // 止盈止损单触发价格
    pub F: String, // 冰山订单数量
    pub g: i64,    // OCO订单 OrderListId
    pub C: String, // 原始订单自定义ID
    pub x: String, // 本次事件的具体执行类型
    pub X: State,  // 订单的当前状态
    pub r: String, // 订单被拒绝的原因
    pub i: i64,    // orderId
    pub l: String, // 订单末次成交量
    pub z: String, // 订单累计已成交量
    pub L: String, // 订单末次成交价格
    pub n: String, // 手续费数量
    pub N: Option<String>, // 手续费资产类别
    pub T: i64,    // 成交时间
    pub I: i64,    // Execution ID
    pub w: bool,   // 订单是否在订单簿上？
    pub m: bool,   // 该成交是作为挂单成交吗？
    pub M: bool,   // 请忽略
    pub O: i64,    // 订单创建时间
    pub Z: String, // 订单累计已成交金额
    pub Y: String, // 订单末次成交金额
    pub Q: String, // Quote Order Quantity
    pub D: Option<i64>, // 追踪时间; 这仅在追踪止损订单已被激活时可见
    pub W: Option<i64>, // Working Time; 订单被添加到 order book 的时间
    pub V: String, // SelfTradePreventionMode

    // 可选字段 - 仅在满足特定条件时出现
    pub d: Option<i64>,      // Trailing Delta - 出现在追踪止损订单中
    pub j: Option<i64>,      // Strategy Id - 如果在请求中添加了strategyId参数
    pub J: Option<i64>,      // Strategy Type - 如果在请求中添加了strategyType参数
    pub v: Option<i64>,      // Prevented Match Id - 只有在因为 STP 导致订单失效时可见
    pub A: Option<String>,   // Prevented Quantity
    pub B: Option<String>,   // Last Prevented Quantity
    pub u: Option<i64>,      // Trade Group Id
    pub U: Option<i64>,      // Counter Order Id
    pub Cs: Option<String>,  // Counter Symbol
    pub pl: Option<String>,  // Prevented Execution Quantity
    pub pL: Option<String>,  // Prevented Execution Price
    pub pY: Option<String>,  // Prevented Execution Quote Qty
    pub b: Option<String>,   // Match Type - 只有在订单有分配时可见
    pub a: Option<i64>,      // Allocation ID
    pub k: Option<String>,   // Working Floor - 只有在订单可能有分配时可见
    pub uS: Option<bool>,    // UsedSor - 只有在订单使用 SOR 时可见
    pub gP: Option<String>,  // Pegged Price Type - 仅出现在挂钩订单中
    pub gOT: Option<String>, // Pegged offset Type
    pub gOV: Option<i64>,    // Pegged Offset Value
    pub gp: Option<String>,  // Pegged Price
}

impl OrderTrait for ExecutionReport {
    fn commission(&self) -> f64 {
        self.n.parse::<f64>().unwrap_or(0.0)
    }
    fn net(&self) -> anyhow::Result<f64> {
        Ok(self.trd_vol()? - self.commission())
    }
    fn side(&self) -> Side {
        self.S
    }
    fn state(&self) -> State {
        self.X
    }
    fn symbol(&self) -> &str {
        self.s.as_str()
    }
    fn trd_vol(&self) -> anyhow::Result<f64> {
        Ok(self.l.parse::<f64>()?)
    }
}

impl From<ExecutionReport> for Order {
    fn from(value: ExecutionReport) -> Self {
        let client_order_id = match value.X {
            State::CANCELED => &value.C,
            _ => &value.c,
        }
        .parse::<u64>()
        .unwrap_or_default();

        let internal_id = client_order_id & 0xFFFFFFFF;
        Self {
            internal_id: internal_id as u32,
            state: value.X,
            order_id: value.i,
            symbol: value.s,
            side: value.S,
            type_: value.o.parse().unwrap(),
            tif: value.f.parse().unwrap(),
            price: value.p.parse().unwrap_or_default(),
            quantity: value.q.parse().unwrap_or_default(),
            trade_time: value.T,
            trade_price: value.L.parse().unwrap_or_default(),
            trade_quantity: value.l.parse().unwrap_or_default(),
            acc: value.z.parse().unwrap_or_default(),
            making: value.m,
        }
    }
}

// usdt

// json! {
//     UsdtListenKey {
//         listenKey: String
//     }
// }

// impl ListenKey for UsdtListenKey {
//     fn key(&self) -> &str {
//         self.listenKey.as_str()
//     }
// }

json! {
    UsdtExpired {
        stream: String,
        data: {
            e: String,
            E: String,
            listenKey: String,
        }
    }
}

json! {
    MarginItem {
        s: String,
        ps: String,
        pa: String,
        mt: String,
        iw: String,
        mp: String,
        up: String,
        mm: String,
    }
}

json! {
    MarginCall {
        E: i64,
        cw: String,
        p: [MarginItem]
    }
}

json! {
    UsdtPosition {
        s: String,
        pa: String,
        ep: String,
        bep: String,
        cr: String,
        up: String,
        mt: String,
        iw: String,
        ps: String
    }
}

json! {
    Asset {
        a: String,
        wb: String,
        cw: String,
        bc: String
    }
}

json! {
    AccountUpdate {
        E: i64,
        T: i64,
        a: {
            m: String,
            B: [Asset],
            P: [Position]
        }
    }
}

json! {
    AccountConfigUpdate {
        E: i64,
        T: i64,
        ac: {
            s: String,
            l: u16
        }
    }
}

json! {
    MultiAssetsAccountConfigUpdate {
        E: i64,
        T: i64,
        ai: {
            j: bool
        }
    }
}

json! {
    StrategyUpdate {
        T: i64,
        E: i64,
        su: {
            si: i64,
            st: String,
            ss: String,
            s: String,
            ut: i64,
            c: i32,
        }
    }
}

json! {
    GridUpdate {
        T: i64,
        E: i64,
        gu: {
            si: i64,
            st: String,
            ss: String,
            s: String,
            r: String,
            up: String,
            uq: String,
            uf: String,
            mp: String,
            ut: i64
        }
    }
}

json! {
    ConditionalOrderTriggerReject {
        E: i64,
        T: i64,
        or: {
            s: String,
            i: i64,
            r: String,
        }
    }
}

json! {
    RiskLevelChange {
        E: i64,
        u: String,
        s: String,
        eq: String,
        ae: String,
        m: String
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(unused)]
pub enum Event {
    Success(Response<Option<i64>>),
    Error(ErrorResponse),
    Stream(MarketStream),
    // spot
    ExecutionReport(ExecutionReport),
    // SpotListenKey(SpotListenKey),
    Balance(BalanceUpdate),
    OutboundAccountPosition(OutboundAccountPosition),
    SpotExpired(SpotExpired),
    ListenStatus(ListenStatus),
    UserLiabilityChange(UserLiabilityChange),
    MarginLevelStatusChange(MarginLevelStatusChange),
    // usdt
    OrderUpdate(OrderUpdate),
    // UsdtListenKey(UsdtListenKey),
    UsdtExpired(UsdtExpired),
    MarginCall(MarginCall),
    UsdtPosition(UsdtPosition),
    AccountUpdate(AccountUpdate),
    AccountConfigUpdate(AccountConfigUpdate),
    MultiAssetsAccountConfigUpdate(MultiAssetsAccountConfigUpdate),
    StrategyUpdate(StrategyUpdate),
    GridUpdate(GridUpdate),
    ConditionalOrderTriggerReject(ConditionalOrderTriggerReject),
    RiskLevelChange(RiskLevelChange),
}

pub fn deserialize_symbol<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(String::deserialize(deserializer)?.to_lowercase())
}

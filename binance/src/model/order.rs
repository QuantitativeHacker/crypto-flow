use cryptoflow::chat::{OrderType, Side, TimeInForce};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceOrder {
    pub id: u32,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub side: Side,
    pub order_type: OrderType,
    pub tif: TimeInForce,
    pub session_id: u16,
}

#[derive(Debug, Deserialize)]
pub struct BinanceCancel {
    pub symbol: String,
    pub session_id: u16,
    pub order_id: u32,
}

pub mod usdt {
    use cryptoflow::chat::{Side, State};
    use serde::{Deserialize, Serialize};

    use super::super::deserialize_symbol;
    use crate::{OrderTrait, SOrder};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[allow(non_snake_case)]
    pub struct OrderData {
        #[serde(deserialize_with = "deserialize_symbol")]
        pub s: String, // 交易对
        pub c: String,          // 客户端自定订单ID
        pub S: Side,            // 订单方向
        pub o: String,          // 订单类型
        pub f: String,          // 有效方式
        pub q: String,          // 订单原始数量
        pub p: String,          // 订单原始价格
        pub ap: String,         // 订单平均价格
        pub sp: String,         // 条件订单触发价格，对追踪止损单无效
        pub x: String,          // 本次事件的具体执行类型
        pub X: State,           // 订单的当前状态
        pub i: i64,             // 订单ID
        pub l: String,          // 订单末次成交量
        pub z: String,          // 订单累计已成交量
        pub L: String,          // 订单末次成交价格
        pub N: Option<String>,  // 手续费资产类型
        pub n: String,          // 手续费数量
        pub T: i64,             // 成交时间
        pub t: i64,             // 成交ID
        pub b: String,          // 买单净值
        pub a: String,          // 卖单净值
        pub m: bool,            // 该成交是作为挂单成交吗？
        pub R: bool,            // 是否是只减仓单
        pub wt: String,         // 触发价类型
        pub ot: String,         // 原始订单类型
        pub ps: String,         // 持仓方向
        pub cp: Option<bool>,   // 是否为触发平仓单; 仅在条件订单情况下会推送此字段
        pub AP: Option<String>, // 追踪止损激活价格, 仅在追踪止损单时会推送此字段
        pub cr: Option<String>, // 追踪止损回调比例, 仅在追踪止损单时会推送此字段
        pub pP: Option<bool>,   // 是否开启条件单触发保护
        pub si: Option<i64>,    // 忽略
        pub ss: Option<i64>,    // 忽略
        pub rp: String,         // 该交易实现盈亏
        pub V: Option<String>,  // 自成交防止模式
        pub pm: Option<String>, // 价格匹配模式
        pub gtd: Option<i64>,   // TIF为GTD的订单自动取消时间
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[serde(into = "SOrder")]
    #[allow(non_snake_case)]
    pub struct OrderUpdate {
        pub e: String,    // 事件类型 "ORDER_TRADE_UPDATE"
        pub E: i64,       // 事件时间
        pub T: i64,       // 撮合时间
        pub o: OrderData, // 订单数据
    }

    impl OrderTrait for OrderUpdate {
        fn commission(&self) -> f64 {
            self.o.n.parse::<f64>().unwrap_or(0.0)
        }
        fn net(&self) -> anyhow::Result<f64> {
            self.trd_vol()
        }
        fn side(&self) -> Side {
            self.o.S
        }
        fn state(&self) -> State {
            self.o.X
        }
        fn symbol(&self) -> &str {
            self.o.s.as_str()
        }
        fn trd_vol(&self) -> anyhow::Result<f64> {
            Ok(self.o.l.parse::<f64>()?)
        }
    }

    impl From<OrderUpdate> for SOrder {
        fn from(value: OrderUpdate) -> Self {
            let o = value.o;
            let client_order_id = o.c.parse::<u64>().unwrap_or_default();
            let internal_id = client_order_id & 0xFFFFFFFF;

            Self {
                internal_id: internal_id as u32,
                state: o.X,
                order_id: o.i,
                symbol: o.s,
                side: o.S,
                order_type: o.o.parse().unwrap(),
                tif: o.f.parse().unwrap(),
                price: o.p.parse().unwrap_or_default(),
                quantity: o.q.parse().unwrap_or_default(),
                trade_time: o.T,
                trade_price: o.L.parse().unwrap_or_default(),
                trade_quantity: o.l.parse().unwrap_or_default(),
                acc: o.z.parse().unwrap_or_default(),
                making: o.m,
            }
        }
    }
}

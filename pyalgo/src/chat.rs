use crate::constant::*;
use binance::model::symbol::BinanceSymbol;
use chrono::DateTime;
use chrono_tz::{Asia::Shanghai, Tz};
use cryptoflow::chat::{ErrorResponse, SLoginResponse, SResponse, Success};
use cryptoflow::trading_rules::TradingRules;
use pyo3::prelude::*;
use pyo3::{conversion::IntoPyObject, IntoPyObjectExt};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
struct Quote {
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[gen_stub_pyclass]
#[pyclass]
pub struct Depth {
    time: u64,
    symbol: String,
    stream: String,
    bids: Vec<Quote>,
    asks: Vec<Quote>,
}

#[gen_stub_pymethods]
#[pymethods]
impl Depth {
    #[getter]
    fn time(&self) -> u64 {
        self.time
    }

    #[getter]
    fn datetime(&self) -> String {
        DateTime::from_timestamp_millis(self.time as i64)
            .unwrap()
            .with_timezone(&Shanghai)
            .to_string()
    }

    #[getter]
    fn symbol(&self) -> &String {
        &self.symbol
    }

    #[getter]
    fn stream(&self) -> &String {
        &self.stream
    }

    #[getter]
    fn bid_level(&self) -> usize {
        self.bids.len()
    }

    #[getter]
    fn ask_level(&self) -> usize {
        self.asks.len()
    }

    fn bid_prc(&self, level: usize) -> f64 {
        match self.bids.get(level) {
            Some(quote) => quote.price,
            None => 0.0,
        }
    }

    fn bid_vol(&self, level: usize) -> f64 {
        match self.bids.get(level) {
            Some(quote) => quote.quantity,
            None => 0.0,
        }
    }

    fn ask_prc(&self, level: usize) -> f64 {
        match self.asks.get(level) {
            Some(quote) => quote.price,
            None => 0.0,
        }
    }

    fn ask_vol(&self, level: usize) -> f64 {
        match self.asks.get(level) {
            Some(quote) => quote.quantity,
            None => 0.0,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[gen_stub_pyclass]
#[pyclass]
pub struct Kline {
    time: u64,       // K线结束时间
    start_time: u64, // K线起始时间
    symbol: String,
    stream: String,
    interval: String,    // K线间隔
    open: f64,           // 开盘价
    high: f64,           // 最高价
    low: f64,            // 最低价
    close: f64,          // 收盘价
    volume: f64,         // 成交量
    amount: f64,         // 成交额
    first_trade_id: i64, // 第一笔成交ID
    last_trade_id: i64,  // 最后一笔成交ID
    trade_count: i64,    // 成交数量
    is_closed: bool,     // K线是否完结
    buy_volume: f64,     // 主动买入成交量
    buy_amount: f64,     // 主动买入成交额
}

#[gen_stub_pymethods]
#[pymethods]
impl Kline {
    #[getter]
    fn time(&self) -> u64 {
        self.time
    }

    #[getter]
    fn datetime(&self) -> String {
        DateTime::from_timestamp_millis(self.time as i64)
            .unwrap()
            .with_timezone(&Shanghai)
            .to_string()
    }

    #[getter]
    fn symbol(&self) -> &String {
        &self.symbol
    }

    #[getter]
    fn stream(&self) -> &String {
        &self.stream
    }

    #[getter]
    fn open(&self) -> f64 {
        self.open
    }

    #[getter]
    fn high(&self) -> f64 {
        self.high
    }

    #[getter]
    fn low(&self) -> f64 {
        self.low
    }

    #[getter]
    fn close(&self) -> f64 {
        self.close
    }

    #[getter]
    fn volume(&self) -> f64 {
        self.volume
    }

    #[getter]
    fn amount(&self) -> f64 {
        self.amount
    }

    #[getter]
    fn start_time(&self) -> u64 {
        self.start_time
    }

    #[getter]
    fn start_datetime(&self) -> String {
        DateTime::from_timestamp_millis(self.start_time as i64)
            .unwrap()
            .with_timezone(&Shanghai)
            .to_string()
    }

    #[getter]
    fn interval(&self) -> &String {
        &self.interval
    }

    #[getter]
    fn first_trade_id(&self) -> i64 {
        self.first_trade_id
    }

    #[getter]
    fn last_trade_id(&self) -> i64 {
        self.last_trade_id
    }

    #[getter]
    fn trade_count(&self) -> i64 {
        self.trade_count
    }

    #[getter]
    fn is_closed(&self) -> bool {
        self.is_closed
    }

    #[getter]
    fn buy_volume(&self) -> f64 {
        self.buy_volume
    }

    #[getter]
    fn buy_amount(&self) -> f64 {
        self.buy_amount
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)] // 自动识别类型
pub enum Product {
    Binance(BinanceSymbol),
    // 未来可以添加其他交易所
    // Okx(OkxInstrument),
}

// 为 Product 枚举实现 TradingRules trait
impl TradingRules for Product {
    fn symbol(&self) -> &String {
        match self {
            Product::Binance(b) => b.symbol(),
            // Product::Okx(o) => o.symbol(),
        }
    }

    fn min_price(&self) -> f64 {
        match self {
            Product::Binance(b) => b.min_price(),
            // Product::Okx(o) => o.min_price(),
        }
    }

    fn max_price(&self) -> f64 {
        match self {
            Product::Binance(b) => b.max_price(),
            // Product::Okx(o) => o.max_price(),
        }
    }

    fn tick_size(&self) -> f64 {
        match self {
            Product::Binance(b) => b.tick_size(),
            // Product::Okx(o) => o.tick_size(),
        }
    }

    fn min_quantity(&self) -> f64 {
        match self {
            Product::Binance(b) => b.min_quantity(),
            // Product::Okx(o) => o.min_quantity(),
        }
    }

    fn max_quantity(&self) -> f64 {
        match self {
            Product::Binance(b) => b.max_quantity(),
            // Product::Okx(o) => o.max_quantity(),
        }
    }

    fn lot_size(&self) -> f64 {
        match self {
            Product::Binance(b) => b.lot_size(),
            // Product::Okx(o) => o.lot_size(),
        }
    }

    fn min_notional(&self) -> f64 {
        match self {
            Product::Binance(b) => b.min_notional(),
            // Product::Okx(o) => o.min_notional(),
        }
    }
}

impl Product {
    pub fn symbol(&self) -> &String {
        match self {
            Product::Binance(inner) => inner.symbol(),
        }
    }

    pub fn delivery(&self) -> DateTime<Tz> {
        match self {
            Product::Binance(b) => match b.deliveryDate {
                Some(delivery) => DateTime::from_timestamp_millis(delivery as i64)
                    .unwrap()
                    .with_timezone(&Shanghai),
                None => DateTime::<Tz>::MAX_UTC.with_timezone(&Shanghai),
            }, // Product::Okx(o) => { /* OKX 实现 */ }
        }
    }

    pub fn onboard(&self) -> DateTime<Tz> {
        match self {
            Product::Binance(b) => match b.onboardDate {
                Some(onboard) => DateTime::from_timestamp_millis(onboard as i64)
                    .unwrap()
                    .with_timezone(&Shanghai),
                None => DateTime::<Tz>::MAX_UTC.with_timezone(&Shanghai),
            }, // Product::Okx(o) => { /* OKX 实现 */ }
        }
    }

    pub fn order_support(&self, order_type: &OrderType) -> bool {
        match self {
            Product::Binance(b) => b.orderTypes.contains(&order_type.to_string()), // Product::Okx(o) => { /* OKX 实现 */ }
        }
    }

    // 使用 TradingRules trait 的方法，更简洁
    pub fn max_prc(&self) -> f64 {
        self.max_price()
    }

    pub fn min_prc(&self) -> f64 {
        self.min_price()
    }

    pub fn tick_size(&self) -> f64 {
        TradingRules::tick_size(self)
    }

    pub fn lot(&self) -> f64 {
        self.lot_size()
    }

    pub fn min_notional(&self) -> f64 {
        TradingRules::min_notional(self)
    }
}

type Products = SResponse<Vec<Product>>;

#[derive(Debug, Deserialize)]
pub struct PositionRsp {
    pub session_id: u16,
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Success(SSuccess),
    Login(SLoginResponse),
    Error(SErrorResponse),
    Depth(Depth),
    Kline(Kline),
    Order(Order),
    Products(Products),
    Positions(SResponse<PositionRsp>),
    Position(Position),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[gen_stub_pyclass_enum]
#[pyclass(eq)]
pub enum EventType {
    Login,
    Depth,
    Kline,
    Order,
    Position,
}

#[derive(Debug)]
#[gen_stub_pyclass]
#[pyclass]
pub struct Event {
    event_type: EventType,
    data: Py<PyAny>,
}

impl Event {
    pub fn new<T: for<'a> IntoPyObject<'a>>(event_type: EventType, data: T) -> Py<PyAny> {
        Python::attach(|py| {
            Self {
                event_type,
                data: data.into_py_any(py).unwrap(),
            }
            .into_py_any(py)
            .unwrap()
        })
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Event {
    #[getter]
    fn event_type(&self) -> EventType {
        self.event_type
    }

    #[getter]
    fn data(&self) -> &Py<PyAny> {
        &self.data
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Deserialize)]
#[gen_stub_pyclass]
#[pyclass]
pub struct Order {
    symbol: String,
    side: Side,
    state: State,
    order_type: OrderType,
    tif: Tif,
    quantity: f64,
    price: f64,
    #[allow(unused)]
    order_id: i64,
    internal_id: u8,
    trade_time: i64,
    trade_price: f64,
    trade_quantity: f64,
    acc: f64,
    making: Option<bool>,
}

impl Order {
    pub fn new(
        id: u8,
        symbol: &str,
        price: f64,
        quantity: f64,
        side: Side,
        order_type: OrderType,
        tif: Tif,
    ) -> Self {
        Self {
            price,
            quantity,
            symbol: symbol.into(),
            side,
            state: State::NEW,
            order_type,
            tif,
            order_id: -1,
            internal_id: id,
            trade_time: 0,
            trade_price: 0.0,
            trade_quantity: 0.0,
            acc: 0.0,
            making: None,
        }
    }

    pub fn on_update(&mut self, other: Self) {
        *self = other
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Order {
    #[getter]
    fn symbol(&self) -> &str {
        &self.symbol
    }
    #[getter]
    fn side(&self) -> Side {
        self.side
    }
    #[getter]
    fn state(&self) -> State {
        self.state
    }
    #[getter]
    fn order_type(&self) -> OrderType {
        self.order_type
    }
    #[getter]
    fn tif(&self) -> Tif {
        self.tif
    }
    #[getter]
    fn quantity(&self) -> f64 {
        self.quantity
    }
    #[getter]
    fn price(&self) -> f64 {
        self.price
    }
    #[getter]
    pub fn id(&self) -> u8 {
        self.internal_id
    }
    #[getter]
    fn trade_time(&self) -> i64 {
        self.trade_time
    }
    #[getter]
    fn trade_dt(&self) -> String {
        DateTime::from_timestamp_millis(self.trade_time as i64)
            .unwrap()
            .with_timezone(&Shanghai)
            .to_string()
    }
    #[getter]
    fn trade_price(&self) -> f64 {
        self.trade_price
    }
    #[getter]
    fn trade_quantity(&self) -> f64 {
        self.trade_quantity
    }
    #[getter]
    fn acc(&self) -> f64 {
        self.acc
    }

    #[getter]
    fn making(&self) -> Option<bool> {
        self.making
    }

    #[getter]
    pub fn is_active(&self) -> bool {
        matches!(self.state, State::NEW | State::PARTIALLY_FILLED)
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }
}

#[derive(Debug, Serialize)]
pub struct OrderRequest {
    pub id: u8,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub side: Side,
    pub order_type: OrderType,
    pub tif: Tif,
    pub session_id: u16,
}

#[derive(Debug, Serialize)]
pub struct CancelRequest {
    pub symbol: String,
    pub session_id: u16,
    pub order_id: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[gen_stub_pyclass]
#[pyclass]
pub struct Position {
    pub symbol: String,
    pub net: f64,
}

#[derive(Debug, Deserialize)]
#[gen_stub_pyclass]
#[pyclass]
#[allow(non_snake_case)]
pub struct PremiumIndex {
    #[serde(deserialize_with = "deserialize_symbol")]
    symbol: String,
    #[serde(deserialize_with = "string_to_f64")]
    markPrice: f64,
    #[serde(deserialize_with = "string_to_f64")]
    indexPrice: f64,
    #[serde(deserialize_with = "string_to_f64")]
    estimatedSettlePrice: f64,
    #[serde(deserialize_with = "string_to_f64")]
    lastFundingRate: f64,
    nextFundingTime: i64,
    #[serde(deserialize_with = "string_to_f64")]
    interestRate: f64,
    time: i64,
}

#[gen_stub_pymethods]
#[pymethods]
impl PremiumIndex {
    #[getter]
    fn time(&self) -> i64 {
        self.time
    }
    #[getter]
    fn datetime(&self) -> String {
        DateTime::from_timestamp_millis(self.time)
            .unwrap()
            .with_timezone(&Shanghai)
            .to_string()
    }
    #[getter]
    fn symbol(&self) -> &str {
        &self.symbol
    }
    #[getter]
    fn mark_price(&self) -> f64 {
        self.markPrice
    }
    #[getter]
    fn index_price(&self) -> f64 {
        self.indexPrice
    }
    #[getter]
    fn estimated_settle_price(&self) -> f64 {
        self.estimatedSettlePrice
    }
    #[getter]
    fn last_funding_rate(&self) -> f64 {
        self.lastFundingRate
    }
    #[getter]
    fn next_funding_time(&self) -> i64 {
        self.nextFundingTime
    }
    #[getter]
    fn next_funding_dt(&self) -> String {
        DateTime::from_timestamp_millis(self.nextFundingTime as i64)
            .unwrap()
            .with_timezone(&Shanghai)
            .to_string()
    }
    #[getter]
    fn interest_rate(&self) -> f64 {
        self.interestRate
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.parse::<f64>().unwrap())
}

fn deserialize_symbol<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(String::deserialize(deserializer)?.to_lowercase())
}

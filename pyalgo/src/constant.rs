use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum};
use serde::{Deserialize, Serialize};

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    AUCTION,
    PRE_OPEN,
    OPEN,
    PRE_CLOSE,
    CLOSE,
    UNDEF,
}

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[doc = "The side of an order"]
pub enum Side {
    #[doc = "Buy side"]
    BUY,
    #[doc = "Sell side"]
    SELL,
}

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum OrderType {
    #[doc = "Limit order"]
    LIMIT,
    #[doc = "Limit maker order"]
    LIMIT_MAKER,
    #[doc = "Market order"]
    MARKET,
    #[doc = "Stop order"]
    STOP,
    #[doc = "Stop market order"]
    STOP_MARKET,
    #[doc = "Stop loss order"]
    STOP_LOSS,
    #[doc = "Stop loss limit order"]
    STOP_LOSS_LIMIT,
    #[doc = "Take profit order"]
    TAKE_PROFIT,
    #[doc = "Take profit limit order"]
    TAKE_PROFIT_LIMIT,
    TAKE_PROFIT_MARKET,
    #[doc = "Trailing stop market order"]
    TRAILING_STOP_MARKET,
}

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Tif {
    #[doc = "Good till cancel"]
    GTC,
    #[doc = "Immediate or cancel"]
    IOC,
    #[doc = "Fill or kill"]
    FOK,
    #[doc = "Good till date"]
    GTX,
    #[doc = "Good till date"]
    GTD,
    #[doc = "Undefined"]
    UNDEF,
}

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum State {
    NEW,
    #[allow(non_camel_case_types)]
    PARTIALLY_FILLED,
    FILLED,
    CANCELED,
    REJECTED,
    EXPIRED,
    #[allow(non_camel_case_types)]
    EXPIRED_IN_MATCH,
    UNDEF,
}

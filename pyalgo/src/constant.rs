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
pub enum Side {
    BUY,
    SELL,
}

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum OrderType {
    LIMIT,
    LIMIT_MAKER,
    MARKET,
    STOP,
    STOP_MARKET,
    STOP_LOSS,
    STOP_LOSS_LIMIT,
    TAKE_PROFIT,
    TAKE_PROFIT_LIMIT,
    TAKE_PROFIT_MARKET,
    TRAILING_STOP_MARKET,
}

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Tif {
    GTC,
    IOC,
    FOK,
    GTX,
    GTD,
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

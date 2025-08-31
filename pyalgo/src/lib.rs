pub mod chat;
pub mod constant;
pub mod phase;
pub mod rest;
pub mod session;
pub mod subscription;
pub mod ws;

use chat::*;
use constant::*;
use phase::TradingPhase;
use pyo3::prelude::*;
use rest::*;
use session::*;
use subscription::Subscription;

#[pymodule]
fn pyalgo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Kline>()?;
    m.add_class::<Depth>()?;
    m.add_class::<Order>()?;
    m.add_class::<Rest>()?;
    m.add_class::<Session>()?;
    m.add_class::<TradingPhase>()?;
    m.add_class::<Phase>()?;
    m.add_class::<Side>()?;
    m.add_class::<OrderType>()?;
    m.add_class::<Tif>()?;
    m.add_class::<State>()?;
    m.add_class::<EventType>()?;
    m.add_class::<Event>()?;
    m.add_class::<Subscription>()?;
    Ok(())
}

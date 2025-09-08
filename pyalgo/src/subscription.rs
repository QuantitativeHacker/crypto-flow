use crate::{chat::Product, phase::TradingPhase, OrderType, Phase, Position};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rust_decimal::prelude::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};

#[gen_stub_pyclass]
#[pyclass]
pub struct Subscription {
    product: Product,
    position: Option<Position>,
    phase: TradingPhase,
}

impl Subscription {
    pub fn new(product: Product) -> Self {
        Self {
            product,
            position: None,
            phase: TradingPhase::default(),
        }
    }

    pub fn on_position(&mut self, position: Position) {
        self.position = Some(position);
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Subscription {
    #[getter]
    pub fn symbol(&self) -> &String {
        self.product.symbol()
    }

    #[getter]
    pub fn delivery(&self) -> String {
        self.product.delivery().to_string()
    }

    #[getter]
    pub fn onboard(&self) -> String {
        self.product.onboard().to_string()
    }

    #[getter]
    pub fn max_prc(&self) -> f64 {
        self.product.max_prc()
    }

    #[getter]
    pub fn min_prc(&self) -> f64 {
        self.product.min_prc()
    }

    #[getter]
    pub fn tick_size(&self) -> f64 {
        self.product.tick_size()
    }

    #[getter]
    pub fn lot(&self) -> f64 {
        self.product.lot()
    }

    #[getter]
    pub fn min_notional(&self) -> f64 {
        self.product.min_notional()
    }

    #[getter]
    fn net(&self) -> f64 {
        match &self.position {
            Some(position) => position.net,
            None => 0.0,
        }
    }

    pub fn order_support(&self, order_type: &OrderType) -> bool {
        self.product.order_support(order_type)
    }

    pub fn floor_to_lot_size(&self, vol: f64) -> f64 {
        let mut vol = Decimal::from_f64(vol).unwrap();
        let lot = Decimal::from_f64(self.lot()).unwrap();

        vol = (vol / lot).floor() * lot;
        vol.to_f64().unwrap()
    }

    pub fn round_price(&self, price: f64) -> f64 {
        let mut price = Decimal::from_f64(price).unwrap();
        let tick_size = Decimal::from_f64(self.tick_size()).unwrap();

        price = (price / tick_size).round() * tick_size;
        price.to_f64().unwrap()
    }

    fn tick_up(&self, price: f64, n: i32) -> f64 {
        self.round_price(price + (self.tick_size() * n as f64))
    }

    fn tick_dn(&self, price: f64, n: i32) -> f64 {
        self.round_price(price - (self.tick_size() * n as f64))
    }

    pub fn add_phase(&mut self, hour: u32, minute: u32, second: u32, phase: Phase) {
        self.phase.add_phase(hour, minute, second, phase)
    }

    pub fn determine(&self, mills: i64) -> Phase {
        self.phase.determine(mills)
    }
}

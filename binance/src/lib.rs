pub mod account;
pub mod app;
pub mod event_handlers;
pub mod handler;
pub mod market;
pub mod model;
pub mod rest;
pub mod session;
pub mod session_manager;
pub mod subscriber;

pub use account::*;
pub use app::*;
pub use handler::*;
pub use market::*;
pub use session::*;
use std::future::Future;
pub use subscriber::*;

use cryptoflow::chat::*;
use cryptoflow::parser::JsonParser;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::sync::mpsc::UnboundedSender;
use tungstenite::Message;

use crate::model::{
    order::{BinanceCancel, BinanceOrder},
    symbol::BinanceSymbol,
};

pub trait Trade {
    fn disconnected(&self) -> bool;
    fn products(&self) -> &HashMap<String, BinanceSymbol>;
    fn get_positions(&self, session_id: u16) -> Option<&HashMap<String, Position>>;
    fn get_products(&mut self) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn process(&mut self) -> impl Future<Output = anyhow::Result<bool>> + Send;
    fn add_order(&mut self, addr: &SocketAddr, order: &BinanceOrder) -> anyhow::Result<()>;
    fn cancel(&mut self, addr: &SocketAddr, cancel: &BinanceCancel) -> anyhow::Result<()>;
    fn handle_close(&mut self, addr: &SocketAddr) -> anyhow::Result<()>;
    fn handle_login(
        &mut self,
        addr: &SocketAddr,
        req: &SRequest<SLogin>,
        tx: &UnboundedSender<Message>,
    ) -> impl Future<Output = anyhow::Result<Option<SError>>> + Send;
    fn handle_subscribe(
        &mut self,
        addr: &SocketAddr,
        req: &SRequest<Vec<String>>,
    ) -> Option<SError>;
    fn validate_symbol(&self, symbol: &str, stream: &str) -> bool;
    fn handle_disconnect(&mut self, addr: &SocketAddr, parser: &JsonParser) -> anyhow::Result<()>;
    fn reply<T: Serialize + Debug>(
        &mut self,
        addr: &SocketAddr,
        id: i64,
        result: T,
    ) -> anyhow::Result<()>;
}

pub trait OrderTrait {
    fn symbol(&self) -> &str;
    fn trd_vol(&self) -> anyhow::Result<f64>;
    fn commission(&self) -> f64;
    fn net(&self) -> anyhow::Result<f64>;
    fn side(&self) -> Side;
    fn state(&self) -> State;
}

// pub trait ListenKey {
//     fn key(&self) -> &str;
// }

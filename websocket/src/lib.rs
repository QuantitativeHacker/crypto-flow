mod auth;
mod channel;
mod client;
mod client_;
mod error;
mod exchange;
mod request;
mod server;
mod utils;

pub use client::WebSocketClient;
pub use error::Error;
pub use server::WebSocketServer;
pub use server::{Connection, TcpStreamReceiver, TcpStreamSender};

use crate::client_::WebsocketClient;
use crate::exchange::{BinanceProtocol, OkxProtocol};

/// 为了兼容现有调用，保留 OkxWebsocketClient 名称作为泛型别名
pub type OkxWebsocketClient = WebsocketClient<OkxProtocol>;
/// 为了兼容现有调用，提供 Binance 客户端别名
pub type BinanceWebsocketClient = WebsocketClient<BinanceProtocol>;

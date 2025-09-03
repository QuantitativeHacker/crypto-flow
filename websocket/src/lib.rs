mod auth;
pub mod channel;
mod client;
mod error;
mod exchange;
mod request;
mod server;
mod utils;

pub use error::Error;
pub use server::WebSocketServer;
pub use server::{Connection, TcpStreamReceiver, TcpStreamSender};

pub use crate::client::WebsocketClient;
pub use crate::exchange::{BinanceProtocol, BinanceWsApiProtocol, OkxProtocol};

pub use crate::auth::Credentials;

/// Okx websocket client
pub type OkxWebsocketClient = WebsocketClient<OkxProtocol>;
/// Binance websocket client
pub type BinanceWebsocketClient = WebsocketClient<BinanceProtocol>;
/// Binance WS-API websocket client
pub type BinanceWsApiWebsocketClient = WebsocketClient<BinanceWsApiProtocol>;

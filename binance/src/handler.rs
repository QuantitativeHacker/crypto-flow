use crate::market::Market;
use crate::model::order::{BinanceCancel, BinanceOrder};
use crate::Trade;
use log::*;
use std::collections::HashMap;
use std::net::SocketAddr;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(windows)]
use tokio::signal::windows::{ctrl_break, ctrl_c};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::Duration;
use tungstenite::Message;
use websocket::Connection;
use xcrypto::chat::{Login, PositionReq, PositionRsp, Request};
use xcrypto::parser::Parser;

pub struct Handler {
    // Python 策略客户端连接：addr -> (to_client_tx, from_client_rx)
    client_channels: HashMap<SocketAddr, (UnboundedSender<Message>, UnboundedReceiver<Message>)>,
    keep_running: bool,
}

impl Handler {
    pub fn new() -> Self {
        Self {
            client_channels: HashMap::default(),
            keep_running: false,
        }
    }

    // 解析来自策略客户端的消息，处理 Ping 并返回 Parser
    fn parse_client_message(&mut self, addr: &SocketAddr, msg: &Message) -> Option<Parser> {
        if let Some((tx, _)) = self.client_channels.get_mut(addr) {
            match &msg {
                Message::Ping(ping) => {
                    if let Err(e) = tx.send(Message::Pong(ping.to_owned())) {
                        error!("{}", e);
                    }
                    return None;
                }
                Message::Text(text) => match Parser::new(&text) {
                    Ok(kind) => return Some(kind),
                    Err(e) => {
                        error!("Invalid request {} from {}({})", msg, addr, e);
                        return None;
                    }
                },
                _ => {
                    debug!("{}", msg);
                    return None;
                }
            }
        }
        None
    }

    // 新的策略客户端连接接入
    fn on_client_connect(&mut self, connection: Connection, market: &mut Market) {
        let (addr, tx, rx) = connection;

        market.handle_connect(&addr, &tx);
        self.client_channels.insert(addr.clone(), (tx.clone(), rx));
    }

    async fn handle_client_login<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &Parser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        if let Some((tx, _)) = self.client_channels.get(addr) {
            let req = parser.decode::<Request<Login>>()?;
            info!("{:?}", req);

            let params = &req.params;
            if params.trading {
                match trade.handle_login(addr, &req, tx).await? {
                    Some(e) => market.reply(addr, req.id, e)?,
                    None => market.handle_login(addr, &req)?,
                }
            } else {
                market.handle_login(addr, &req)?;
            }
        }

        Ok(())
    }

    async fn handle_client_subscribe<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &Parser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let mut req = parser.decode::<Request<Vec<String>>>()?;
        info!("{:?}", req);

        match trade.handle_subscribe(addr, &mut req) {
            Some(e) => market.reply(addr, req.id, e)?,
            None => market.handle_subscribe(addr, &mut req).await?,
        }
        Ok(())
    }

    fn handle_client_get_products<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &Parser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let req = parser.decode::<Request<Vec<String>>>()?;
        info!("{:?}", req);

        let products = trade.products();
        if req.params.is_empty() {
            let params: Vec<_> = products.values().cloned().collect();
            market.reply(addr, req.id, params)?;
        } else {
            let mut params = vec![];
            for product in products.values() {
                params.push(product);
            }
            market.reply(addr, req.id, params)?;
        }

        Ok(())
    }

    fn handle_client_get_positions<T: Trade>(
        &self,
        addr: &SocketAddr,
        parser: &Parser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let req: Request<PositionReq> = parser.decode()?;
        info!("{:?}", req);

        let params = req.params;
        let session_id = params.session_id;
        let symbols = params.symbols;

        match trade.get_positions(session_id) {
            Some(positions) => {
                let params = if symbols.is_empty() {
                    let params: Vec<_> = positions.values().cloned().collect();
                    PositionRsp {
                        session_id,
                        positions: params,
                    }
                } else {
                    let mut params = Vec::new();
                    for symbol in symbols.iter() {
                        if let Some(position) = positions.get(symbol) {
                            params.push(position.clone());
                        }
                    }
                    PositionRsp {
                        session_id,
                        positions: params,
                    }
                };
                market.reply(addr, req.id, params)?;
            }
            None => market.reply(
                addr,
                req.id,
                PositionRsp {
                    session_id,
                    positions: Vec::new(),
                },
            )?,
        }

        Ok(())
    }

    #[allow(unused)]
    async fn handle_client_order<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &Parser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let req = parser.decode::<Request<BinanceOrder>>()?;
        info!("{:?}", req);

        trade.add_order(addr, &req.params)
    }

    #[allow(unused)]
    async fn handle_client_cancel<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &Parser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let req = parser.decode::<Request<BinanceCancel>>()?;
        info!("{:?}", req);

        trade.cancel(addr, &req.params)
    }

    // 将策略客户端的请求分发给 market / trade
    async fn dispatch_client_request<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: Parser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        if market.disconnected() {
            return market.handle_disconnect(addr, &parser);
        }

        if trade.disconnected() {
            return trade.handle_disconnect(addr, &parser);
        }

        if let Some(val) = parser.get("method") {
            if let Some(method) = val.as_str() {
                match method {
                    "login" => {
                        self.handle_client_login(addr, &parser, market, trade)
                            .await?
                    }
                    "subscribe" => {
                        self.handle_client_subscribe(addr, &parser, market, trade)
                            .await?
                    }
                    "get_products" => {
                        self.handle_client_get_products(addr, &parser, market, trade)?
                    }
                    "get_positions" => {
                        self.handle_client_get_positions(addr, &parser, market, trade)?
                    }
                    "order" => {
                        self.handle_client_order(addr, &parser, market, trade)
                            .await?
                    }
                    "cancel" => {
                        self.handle_client_cancel(addr, &parser, market, trade)
                            .await?
                    }
                    _ => (),
                }
            }
        }
        Ok(())
    }

    // 主循环：接入策略客户端，处理交易所数据与策略请求
    pub async fn process<T: Trade>(
        &mut self,
        mut client_conn_rx: UnboundedReceiver<Connection>,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        self.keep_running = true;
        #[cfg(unix)]
        let mut terminate = signal(SignalKind::terminate())?;
        #[cfg(unix)]
        let mut interrupt = signal(SignalKind::interrupt())?;
        #[cfg(windows)]
        let mut terminate = ctrl_c()?;
        #[cfg(windows)]
        let mut interrupt = ctrl_break()?;

        while self.keep_running {
            tokio::select! {
               Some(connection) = client_conn_rx.recv() => self.on_client_connect(connection,market),
               res = market.process() => {
                    if let Err(e) = res {
                        error!("{}", e);
                    }
               },
               Some(_) = interrupt.recv() => self.stop(),
               Some(_) = terminate.recv() => self.stop(),
               _ = tokio::time::sleep(Duration::ZERO) => ()
            }

            // avoid two mutable borrow of trade
            tokio::select! {
                res = trade.process() => {
                    if let Err(e) = res {
                        error!("{}", e);
                    }
                },
                _ = tokio::time::sleep(Duration::ZERO) => ()
            }

            let list: Vec<_> = self
                .client_channels
                .iter_mut()
                .map(|(addr, (_, rx))| (addr.clone(), rx.try_recv(), rx.is_closed()))
                .collect();

            for (addr, result, is_closed) in list {
                match &result {
                    Ok(msg) => match msg {
                        Message::Close(_) => {
                            if let Err(e) = self.prune(&addr, market, trade).await {
                                error!("{}", e);
                            }
                        }
                        _ => {
                            if let Some(req) = self.parse_client_message(&addr, msg) {
                                if let Err(e) = self
                                    .dispatch_client_request(&addr, req, market, trade)
                                    .await
                                {
                                    error!("{}, {}", e, msg);
                                }
                            }
                        }
                    },
                    Err(_) => {
                        if is_closed {
                            if let Err(e) = self.prune(&addr, market, trade).await {
                                error!("{}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn prune<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        self.client_channels.remove(addr);
        market.handle_close(addr).await?;
        trade.handle_close(addr)?;

        Ok(())
    }

    pub fn stop(&mut self) {
        info!("Handler stop process");
        self.keep_running = false;
    }
}

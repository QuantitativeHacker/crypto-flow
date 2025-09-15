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

use cryptoflow::chat::{SLogin, SPositionReq, SPositionRsp, SRequest};
use cryptoflow::parser::JsonParser;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::Duration;
use tungstenite::Message;
use websocket::Connection;

/// 客户端方法枚举
#[derive(Debug, Clone, Copy)]
enum ClientMethod {
    Login,
    Subscribe,
    GetProducts,
    GetPositions,
    Order,
    Cancel,
}

impl ClientMethod {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "login" => Some(Self::Login),
            "subscribe" => Some(Self::Subscribe),
            "get_products" => Some(Self::GetProducts),
            "get_positions" => Some(Self::GetPositions),
            "order" => Some(Self::Order),
            "cancel" => Some(Self::Cancel),
            _ => None,
        }
    }
}

pub struct Handler {
    /// Python 策略客户端连接：addr -> (to_client_tx, from_client_rx)
    /// 可以收发消息
    strategy_client_channels:
        HashMap<SocketAddr, (UnboundedSender<Message>, UnboundedReceiver<Message>)>,
    keep_running: bool,
}

impl Handler {
    pub fn new() -> Self {
        Self {
            strategy_client_channels: HashMap::default(),
            keep_running: false,
        }
    }

    // 新的策略客户端连接接入
    fn on_strategy_client_connect(&mut self, connection: Connection, market: &mut Market) {
        let (addr, tx, rx) = connection;
        market.handle_strategy_client_connect(&addr, &tx);
        self.strategy_client_channels.insert(addr.clone(), (tx, rx));
    }

    async fn handle_strategy_client_login<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        if let Some((tx, _)) = self.strategy_client_channels.get(addr) {
            let req = parser.decode::<SRequest<SLogin>>()?;
            info!("{:?}", req);

            let params = &req.params;
            if params.trading {
                match trade.handle_strategy_client_login(addr, &req, tx).await? {
                    Some(e) => trade.reply(addr, req.id, e)?,
                    None => {}
                }
            }
            market.handle_strategy_client_login(addr, &req)?;
        }

        Ok(())
    }

    async fn handle_strategy_client_subscribe<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let mut req = parser.decode::<SRequest<Vec<String>>>()?;
        info!("{:?}", req);

        match trade.handle_strategy_client_subscribe(addr, &mut req) {
            Some(e) => market.reply_to_strategy_client(addr, req.id, e)?,
            None => {
                market
                    .handle_strategy_client_subscribe(addr, &mut req)
                    .await?
            }
        }
        Ok(())
    }

    fn handle_strategy_client_get_products<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let req = parser.decode::<SRequest<Vec<String>>>()?;

        let products = trade.products();
        if req.params.is_empty() {
            let params: Vec<_> = products.values().cloned().collect();
            market.reply_to_strategy_client(addr, req.id, params)?;
        } else {
            let mut params = vec![];
            for product in products.values() {
                params.push(product);
            }
            market.reply_to_strategy_client(addr, req.id, params)?;
        }

        Ok(())
    }

    fn handle_strategy_client_get_positions<T: Trade>(
        &self,
        addr: &SocketAddr,
        parser: &JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let req: SRequest<SPositionReq> = parser.decode()?;
        info!("{:?}", req);

        let params = req.params;
        let session_id = params.session_id;
        let symbols = params.symbols;

        match trade.get_positions(session_id) {
            Some(positions) => {
                let params = if symbols.is_empty() {
                    let params: Vec<_> = positions.values().cloned().collect();
                    SPositionRsp {
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
                    SPositionRsp {
                        session_id,
                        positions: params,
                    }
                };
                market.reply_to_strategy_client(addr, req.id, params)?;
            }
            None => market.reply_to_strategy_client(
                addr,
                req.id,
                SPositionRsp {
                    session_id,
                    positions: Vec::new(),
                },
            )?,
        }

        Ok(())
    }

    #[allow(unused)]
    async fn handle_strategy_client_order<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let req = parser.decode::<SRequest<BinanceOrder>>()?;
        info!("recv Order {:?}", req);

        trade.add_order(addr, &req.params)
    }

    #[allow(unused)]
    async fn handle_strategy_client_cancel<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let req = parser.decode::<SRequest<BinanceCancel>>()?;
        info!("{:?}", req);

        trade.cancel(addr, &req.params)
    }

    // 解析来自策略客户端的消息， Parser
    fn parse_strategy_client_message(
        &mut self,
        addr: &SocketAddr,
        msg: &Message,
    ) -> Option<JsonParser> {
        let Message::Text(text) = msg else {
            warn!("Invalid message {} from strategy client {}", msg, addr);
            return None;
        };

        JsonParser::new(text)
            .map_err(|e| {
                error!("Invalid request {} from {}({})", msg, addr, e);
            })
            .ok()
    }

    // 将策略客户端的请求分发给 market / trade
    async fn dispatch_strategy_client_request<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        // 检查连接状态
        self.check_connection_status(addr, &parser, market, trade)?;

        // 解析并处理方法
        self.handle_client_method(addr, &parser, market, trade)
            .await
    }

    /// 检查market和trade的连接状态
    fn check_connection_status<T: Trade>(
        &self,
        addr: &SocketAddr,
        parser: &JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        if market.disconnected() {
            return market.handle_strategy_client_disconnect(addr, parser);
        }
        if trade.disconnected() {
            return trade.handle_strategy_client_disconnect(addr, parser);
        }
        Ok(())
    }

    /// 处理客户端方法调用
    async fn handle_client_method<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        parser: &JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        let method = parser
            .get("method")
            .and_then(|v| v.as_str())
            .and_then(ClientMethod::from_str);

        if let Some(method) = method {
            self.execute_client_method(method, addr, parser, market, trade)
                .await?
        }

        Ok(())
    }

    /// 执行具体的客户端方法
    async fn execute_client_method<T: Trade>(
        &mut self,
        method: ClientMethod,
        addr: &SocketAddr,
        parser: &JsonParser,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        match method {
            ClientMethod::Login => {
                self.handle_strategy_client_login(addr, parser, market, trade)
                    .await
            }
            ClientMethod::Subscribe => {
                self.handle_strategy_client_subscribe(addr, parser, market, trade)
                    .await
            }
            ClientMethod::GetProducts => {
                self.handle_strategy_client_get_products(addr, parser, market, trade)
            }
            ClientMethod::GetPositions => {
                self.handle_strategy_client_get_positions(addr, parser, market, trade)
            }
            ClientMethod::Order => {
                self.handle_strategy_client_order(addr, parser, market, trade)
                    .await
            }
            ClientMethod::Cancel => {
                self.handle_strategy_client_cancel(addr, parser, market, trade)
                    .await
            }
        }
    }

    // 小批量清理/处理各客户端消息，提升吞吐与公平性
    async fn drain_strategy_client_messages<T: Trade>(
        &mut self,
        market: &mut Market,
        trade: &mut T,
    ) {
        // 先收集，后处理，避免在借用 client_channels 时调用 &mut self 的异步方法造成可变借用冲突
        let mut batch: Vec<(
            SocketAddr,
            Result<Message, tokio::sync::mpsc::error::TryRecvError>,
            bool,
        )> = Vec::new();

        for (addr, (_, rx)) in self.strategy_client_channels.iter_mut() {
            // 一次最多处理MAX_CLIENT_MSG_BATCH个
            let mut cnt = 0usize;
            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        batch.push((addr.clone(), Ok(msg), false));
                        cnt += 1;
                        if cnt >= MAX_CLIENT_MSG_BATCH {
                            break;
                        }
                    }
                    Err(e) => {
                        // 如果通道被关闭，记录以便后续 prune
                        if rx.is_closed() {
                            batch.push((addr.clone(), Err(e), true));
                        }
                        break;
                    }
                }
            }
        }

        for (addr, result, is_closed) in batch {
            match result {
                Ok(msg) => match msg {
                    Message::Close(_) => {
                        if let Err(e) = self.prune(&addr, market, trade).await {
                            error!("{}", e);
                        }
                    }
                    _ => {
                        // 成功接收，那么解析消息并处理
                        if let Some(req) = self.parse_strategy_client_message(&addr, &msg) {
                            if let Err(e) = self
                                .dispatch_strategy_client_request(&addr, req, market, trade)
                                .await
                            {
                                error!("Dispatch client request failed, err:{}, msg: {}", e, msg);
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

    // 主循环：
    // 1. 处理exchange发来的消息并转发
    // 2. 处理client的消息，处理后发送给exchange
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

        // 定期唤醒：即使没有其他事件，也能按节拍清理/处理客户端消息
        let mut tick = tokio::time::interval(Duration::from_millis(1));

        while self.keep_running {
            tokio::select! {
                // 新的客户端连接
                Some(connection) = client_conn_rx.recv() => {
                    self.on_strategy_client_connect(connection, market);
                },
                // 系统信号
                Some(_) = interrupt.recv() => {
                    self.stop();
                },
                Some(_) = terminate.recv() => {
                    self.stop();
                },
                // 市场数据推进
                res = market.process() => {
                    if let Err(e) = res { error!("{}", e); }
                },
                // 账户/交易数据推进
                res = trade.process() => {
                    if let Err(e) = res { error!("{}", e); }
                },
                // 定时唤醒，用于在空闲时也能及时处理客户端消息
                _ = tick.tick() => {
                    // no-op; fallthrough to draining below
                },
            }

            // 每轮 select 后，批量处理各客户端队列中的消息
            self.drain_strategy_client_messages(market, trade).await;
        }

        Ok(())
    }

    async fn prune<T: Trade>(
        &mut self,
        addr: &SocketAddr,
        market: &mut Market,
        trade: &mut T,
    ) -> anyhow::Result<()> {
        self.strategy_client_channels.remove(addr);
        market.handle_strategy_client_close(addr).await?;
        trade.handle_strategy_client_close(addr)?;

        Ok(())
    }

    pub fn stop(&mut self) {
        info!("Handler stop process");
        self.keep_running = false;
    }
}

const MAX_CLIENT_MSG_BATCH: usize = 16;

use super::handler::Handler;
use crate::market::Market; // 交易所（Binance）交互
use crate::Trade; // 交易逻辑（撮合/下单接口）

use log::*;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tungstenite::Message;
use websocket::{Connection, TcpStreamReceiver, TcpStreamSender, WebSocketServer}; // 与 Python 客户端的 WS 服务器

pub struct Application {
    listener: WebSocketServer,
}

impl Application {
    pub async fn new(local: &str) -> anyhow::Result<Self> {
        info!("-------------------- Start --------------------");
        let listener = WebSocketServer::new(local).await?;
        Ok(Self { listener })
    }

    /// 接收“策略客户端（Python）⇄本系统”的 WebSocket 连接，并把连接交给 handler
    async fn accept_strategy_clients(
        &self,
        client_conn_tx: &UnboundedSender<Connection>,
        mut stop: oneshot::Receiver<()>,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                res = self.listener.accept() => {
                    match res {
                        Ok((addr, client_write, client_read)) => {
                            // to_handler: 客户端请求 -> handler；from_handler: handler 响应 -> 客户端
                            let (to_handler_tx, to_handler_rx) = unbounded_channel();
                            let (from_handler_tx, from_handler_rx) = unbounded_channel();

                            // 通知 handler 有新连接
                            client_conn_tx.send((addr, from_handler_tx, to_handler_rx))?;

                            // 为该客户端启动 IO 转发任务
                            tokio::spawn(client_io_task(to_handler_tx, from_handler_rx, client_write, client_read));
                        },
                        Err(e) => error!("{}", e)
                    }

                }
                _ = &mut stop => break,
            }
        }

        Ok(())
    }

    pub async fn keep_running<T: Trade + Send + 'static>(
        self,
        mut market: Market,
        mut trade: T,
    ) -> anyhow::Result<()> {
        let (client_conn_tx, client_conn_rx) = unbounded_channel();
        let (stop_tx, stop_rx) = oneshot::channel();

        tokio::spawn(async move {
            let mut handler = Handler::new();

            if let Err(e) = handler
                .process(client_conn_rx, &mut market, &mut trade)
                .await
            {
                error!("{}", e);
            }

            info!("-------------------- Exit --------------------");
            let _ = stop_tx.send(());
        });

        // run strategy clients
        self.accept_strategy_clients(&client_conn_tx, stop_rx)
            .await?;
        Ok(())
    }
}

// 从“客户端读”并转发给 handler（Python -> Server）
async fn handle_client_message(
    client_read: &mut TcpStreamReceiver,
    to_handler_tx: &UnboundedSender<Message>,
) -> anyhow::Result<()> {
    if let Some(inner) = client_read.recv().await {
        let msg = inner?;
        match msg {
            Message::Close(_) => {
                info!("Peer {} Close", client_read.addr());
                to_handler_tx.send(msg)?;
                return Err(anyhow::anyhow!("WebSocket Close"));
            }
            _ => to_handler_tx.send(msg)?,
        }
    }

    Ok(())
}

// 从 handler 收到响应并发给“客户端写”（Server -> Python）
async fn handle_client_response(
    from_handler_rx: &mut UnboundedReceiver<Message>,
    client_write: &mut TcpStreamSender,
) -> anyhow::Result<()> {
    match from_handler_rx.recv().await {
        Some(inner) => client_write.send(inner).await?,
        None => {
            if from_handler_rx.is_closed() {
                return Err(anyhow::anyhow!("Receiver Close"));
            }
        }
    }
    Ok(())
}

// 单个客户端的 IO 转发任务
async fn client_io_task(
    to_handler_tx: UnboundedSender<Message>,
    mut from_handler_rx: UnboundedReceiver<Message>,
    mut client_write: TcpStreamSender,
    mut client_read: TcpStreamReceiver,
) {
    loop {
        tokio::select! {
            res = handle_client_message(&mut client_read, &to_handler_tx) => {
                if let Err(e) = res {
                    error!("{}", e);
                    break
                }
            },
            res = handle_client_response(&mut from_handler_rx, &mut client_write) => {
                if let Err(e) = res {
                    error!("{}", e);
                    break
                }
            }
        }
    }
    info!("{} Task Finish", client_write.addr());
}

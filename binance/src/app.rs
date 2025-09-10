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
    /// 等待accept信号或者stop信号
    /// 当addr地址（往往是8111）通过accept收到新链接的时候
    async fn accept_strategy_clients(
        &self,
        client_conn_tx: &UnboundedSender<Connection>,
        mut stop: oneshot::Receiver<()>,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                res = self.listener.accept() => {
                    match res {
                        Ok((addr, client_sender, client_receiver)) => {
                            // to_handler: 客户端请求 -> handler；from_handler: handler 响应 -> 客户端
                            let (to_handler_tx, to_handler_rx) = unbounded_channel();
                            let (from_handler_tx, from_handler_rx) = unbounded_channel();

                            // 通知 handler 有新连接，把新链接信息发送给链接处理handler
                            // from_handler_tx: sender[handler -> 策略端]
                            // to_handler_rx: receiver[策略端 -> handler]
                            client_conn_tx.send((addr, from_handler_tx, to_handler_rx))?;

                            // 为该策略新链接启动转发任务
                            tokio::spawn(manage_connection_with_strategy(to_handler_tx, from_handler_rx, client_sender, client_receiver));
                        },
                        Err(e) => error!("Accept new connection error: {}", e)
                    }

                }
                _ = &mut stop => break,
            }
        }

        Ok(())
    }

    /// 这个函数是用来运行整个后端的
    /// 它会等待策略端的链接，策略端会把会话的writer和reader发送过来，然后
    pub async fn keep_running<T: Trade + Send + 'static>(
        self,
        mut market: Market,
        mut trade: T,
    ) -> anyhow::Result<()> {
        // 当有新的策略客户端连接时，client_conn_tx会把链接的信息发送给client_conn_rx，即handler
        let (client_conn_tx, client_conn_rx) = unbounded_channel();
        // 当handler出错，也终止接收新的client连接
        let (stop_tx, stop_rx) = oneshot::channel();

        tokio::spawn(async move {
            let mut handler = Handler::new();

            // client_conn_rx是接收链接信息的，里面包含了收发通道
            if let Err(e) = handler
                .process(client_conn_rx, &mut market, &mut trade)
                .await
            {
                error!("Handler process error: {}", e);
            }

            info!("-------------------- Exit --------------------");
            let _ = stop_tx.send(());
        });

        // 接收策略端的链接，进行消息转发。
        self.accept_strategy_clients(&client_conn_tx, stop_rx)
            .await?;
        Ok(())
    }
}
// 从客户端接收消息并转发给服务端处理器
async fn forward_client_to_server(
    client_receiver: &mut TcpStreamReceiver,
    to_handler_tx: &UnboundedSender<Message>,
) -> anyhow::Result<()> {
    if let Some(inner) = client_receiver.recv().await {
        let msg = inner?;
        match msg {
            Message::Close(_) => {
                info!("Peer {} Close", client_receiver.addr());
                to_handler_tx.send(msg)?;
                return Err(anyhow::anyhow!("WebSocket Close"));
            }
            _ => to_handler_tx.send(msg)?,
        }
    }

    Ok(())
}

// 从服务端处理器接收响应并转发给客户端
async fn forward_server_to_client(
    from_handler_rx: &mut UnboundedReceiver<Message>,
    client_sender: &mut TcpStreamSender,
) -> anyhow::Result<()> {
    match from_handler_rx.recv().await {
        Some(inner) => client_sender.send(inner).await?,
        None => {
            if from_handler_rx.is_closed() {
                return Err(anyhow::anyhow!("Receiver Close"));
            }
        }
    }
    Ok(())
}

// 管理单个客户端的双向消息转发
async fn manage_connection_with_strategy(
    to_server_tx: UnboundedSender<Message>,
    mut from_server_rx: UnboundedReceiver<Message>,
    mut client_sender: TcpStreamSender,
    mut client_receiver: TcpStreamReceiver,
) {
    loop {
        tokio::select! {
            res = forward_client_to_server(&mut client_receiver, &to_server_tx) => {
                if let Err(e) = res {
                    error!("{}", e);
                    break
                }
            },
            res = forward_server_to_client(&mut from_server_rx, &mut client_sender) => {
                if let Err(e) = res {
                    error!("{}", e);
                    break
                }
            }
        }
    }
    info!("{} Task Finish", client_sender.addr());
}

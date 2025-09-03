use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::Stream;
use futures::{SinkExt, StreamExt};
use serde_json::{Map, Value, json};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::{Bytes, Error as WsError, Utf8Bytes};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::auth::Credentials;
use crate::channel::{Args, ChannelType};
use crate::error::Error;
use crate::exchange::{WsEndpoints, WsProtocol};
use crate::request::OkxSubscription;

/// 协议无关的本地订阅存根
#[derive(Clone)]
pub struct StoredSub {
    /// 用于内部订阅表的唯一键
    pub key: String,
    /// 协议本地化的订阅对象（OKX 需要；Binance 不需要）
    pub local: Option<OkxSubscription>,
    /// 订阅请求（上行原始 JSON）
    pub req_sub: serde_json::Value,
    /// 退订请求（上行原始 JSON）
    pub req_unsub: serde_json::Value,
}

/// 通用 WebSocket 客户端，协议由策略决定
pub struct WebsocketClient<P: WsProtocol + Clone + Send + Sync + 'static> {
    /// WebSocket连接URL
    url: String,
    /// 是否使用私有WS (需要认证)
    is_private: bool,
    /// 认证凭证
    credentials: Option<Credentials>,
    /// 是否使用模拟交易
    is_simulated: String,
    /// 已订阅的频道（按协议保存足够信息以便重连恢复与退订）
    subscriptions: Arc<Mutex<HashMap<String, StoredSub>>>,
    /// 消息发送通道
    tx: Option<Sender<Message>>,
    /// 数据接收通道
    rx: Option<Receiver<serde_json::Value>>,
    /// 连接任务句柄
    connection_task: Option<JoinHandle<()>>,
    /// 重连任务句柄
    reconnect_task: Option<JoinHandle<()>>,
    /// 最后一次ping时间
    last_ping_time: Arc<Mutex<Instant>>,
    /// 协议策略
    protocol: P,
}

impl<P> WebsocketClient<P>
where
    P: WsProtocol + WsEndpoints + Default + Clone + Send + Sync + 'static,
{
    /// 创建新的公共WebSocket客户端
    pub fn new_public() -> Self {
        Self {
            url: P::default_public_url().to_string(),
            is_private: false,
            credentials: None,
            is_simulated: "0".to_string(),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            tx: None,
            rx: None,
            connection_task: None,
            reconnect_task: None,
            last_ping_time: Arc::new(Mutex::new(Instant::now())),
            protocol: P::default(),
        }
    }

    /// 创建新的私有WebSocket客户端
    pub fn new_private(credentials: Credentials) -> Self {
        let url = P::default_private_url()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                warn!("没有设置私有WebSocket URL，使用公共WebSocket URL");
                P::default_public_url().to_string()
            });
        Self {
            url,
            is_private: true,
            credentials: Some(credentials),
            is_simulated: "0".to_string(),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            tx: None,
            rx: None,
            connection_task: None,
            reconnect_task: None,
            last_ping_time: Arc::new(Mutex::new(Instant::now())),
            protocol: P::default(),
        }
    }

    /// 设置是否使用模拟交易
    pub fn set_simulated_trading(&mut self, is_simulated: String) {
        self.is_simulated = is_simulated;
    }

    /// 设置WebSocket URL
    pub fn set_url(&mut self, url: impl Into<String>) {
        self.url = url.into();
    }

    /// 连接到WebSocket服务器
    pub async fn connect(&mut self) -> Result<Receiver<serde_json::Value>, Error> {
        let url_string = self.url.clone();
        let url = Url::parse(&url_string)
            .map_err(|e| Error::WebSocketError(format!("无效的WebSocket URL: {}", e)))?;

        let (ws_stream, _) = connect_async(url.as_str())
            .await
            .map_err(|e| Error::WebSocketError(format!("连接WebSocket失败: {}", e)))?;

        info!("已连接到WebSocket服务器");

        let (write, read) = ws_stream.split();
        let (tx_in, rx_in) = mpsc::channel::<Message>(100);
        let (tx_out, rx_out) = mpsc::channel::<serde_json::Value>(100);

        // 消息发送任务
        let tx_forward = tokio::spawn(async move {
            let mut rx_in = rx_in;
            let mut write = write;
            while let Some(msg) = rx_in.recv().await {
                if let Err(e) = write.send(msg).await {
                    error!("发送WebSocket消息错误: {}", e);
                    break;
                }
            }
            debug!("WebSocket发送任务结束");
        });

        // 消息接收+心跳任务
        let ping_text = self.protocol.ping_text();
        let rx_task = tokio::spawn(Self::run_ws_with_heartbeat(
            read,
            tx_out.clone(),
            tx_in.clone(),
            self.last_ping_time.clone(),
            Duration::from_secs(15),
            ping_text,
        ));

        // 合并任务
        self.connection_task = Some(tokio::spawn(async move {
            let _ = tokio::join!(tx_forward, rx_task);
            debug!("WebSocket连接任务已结束");
        }));

        self.tx = Some(tx_in);
        self.rx = Some(rx_out);

        // 如果是私有连接，进行认证
        if self.is_private {
            if let Some(ref credentials) = self.credentials {
                if let Some(login_msg) = self.protocol.build_login(credentials) {
                    self.send_raw_json(login_msg).await?;
                    info!("已发送WebSocket登录请求");
                    sleep(Duration::from_millis(500)).await;
                }
            } else {
                return Err(Error::AuthenticationError(
                    "私有WebSocket连接需要凭证".to_string(),
                ));
            }
        }

        // 启动重连任务
        self.start_reconnect_task();

        // 重新订阅现有通道
        let subscriptions_clone = self
            .subscriptions
            .lock()
            .map_err(|_| Error::WebSocketError("获取订阅锁失败".to_string()))?
            .clone();

        for stored in subscriptions_clone.values() {
            // 按协议重放订阅请求
            self.send_raw_json(stored.req_sub.clone()).await?;
        }

        // 直接返回 self.rx.take()，不再转发
        let rx = self
            .rx
            .take()
            .ok_or_else(|| Error::WebSocketError("rx 不存在".to_string()))?;
        Ok(rx)
    }

    /// 关闭连接
    pub async fn close(&mut self) {
        // 发送关闭消息
        if let Some(tx) = &self.tx {
            let _ = tx.send(Message::Close(None)).await;
        }

        // 取消任务
        if let Some(handle) = self.connection_task.take() {
            handle.abort();
        }
        if let Some(handle) = self.reconnect_task.take() {
            handle.abort();
        }

        // 清理资源
        self.tx = None;
        self.rx = None;

        info!("已关闭WebSocket连接");
    }

    /// WS-API: 发送任意请求（不签名）
    pub async fn wsapi_call(&self, method: &str, params: Value, id: i64) -> Result<(), Error> {
        let req = json!({ "id": id, "method": method, "params": params });
        self.send_raw_json(req).await
    }

    /// WS-API: 发送需要签名的请求（Ed25519，自动添加 apiKey/timestamp/signature）
    pub async fn wsapi_call_signed(
        &self,
        method: &str,
        mut params: Map<String, Value>,
        id: i64,
    ) -> Result<(), Error> {
        let cred = match &self.credentials {
            Some(c) => c,
            None => {
                return Err(Error::AuthenticationError(
                    "缺少凭证，无法签名 WS-API 请求".to_string(),
                ));
            }
        };

        // 准备基础参数
        let timestamp = crate::utils::generate_timestamp_websocket();
        params.insert("apiKey".to_string(), Value::String(cred.api_key.clone()));
        params.insert(
            "timestamp".to_string(),
            Value::Number(timestamp.parse::<i64>().unwrap_or(0).into()),
        );

        // 构造 payload: 按 key 字母序，k=v，用 & 连接
        let mut pairs: Vec<(String, String)> = params
            .iter()
            .map(|(k, v)| {
                let vs = match v {
                    Value::String(s) => s.clone(),
                    _ => v.to_string(),
                };
                (k.clone(), vs)
            })
            .collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        let payload = pairs
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        // 签名（cred.api_secret 为 Ed25519 PEM 或其路径）
        let signature = crate::utils::sign_ed25519_base64(&cred.api_secret, &payload)
            .map_err(|e| Error::AuthenticationError(format!("签名失败: {}", e)))?;
        params.insert("signature".to_string(), Value::String(signature));

        let req = Value::Object({
            let mut m = Map::new();
            m.insert("id".to_string(), Value::Number(id.into()));
            m.insert("method".to_string(), Value::String(method.to_string()));
            m.insert("params".to_string(), Value::Object(params));
            m
        });
        self.send_raw_json(req).await
    }

    /// 订阅通道
    pub async fn subscribe(&self, channel: ChannelType, args: Args) -> Result<(), Error> {
        let stored = self.protocol.build_subscribe(channel, &args);
        let key = stored.key.clone();
        if let Ok(mut subscriptions) = self.subscriptions.lock() {
            subscriptions.insert(key, stored.clone());
        } else {
            return Err(Error::WebSocketError("获取订阅锁失败".to_string()));
        }
        self.send_raw_json(stored.req_sub).await
    }

    /// 使用订阅对象进行订阅
    // 保留占位（OKX 重放时已通过 StoredSub 中的 req_sub 实现）

    /// 取消订阅
    pub async fn unsubscribe(&self, channel: ChannelType, args: Args) -> Result<(), Error> {
        let key = self.protocol.make_key(&channel, &args);
        let stored = if let Ok(mut subscriptions) = self.subscriptions.lock() {
            subscriptions.remove(&key)
        } else {
            return Err(Error::WebSocketError("获取订阅锁失败".to_string()));
        };
        if let Some(stored) = stored {
            self.send_raw_json(stored.req_unsub).await
        } else {
            // 未找到，按协议临时构造一次订阅，并取其中的退订报文
            let adhoc = self.protocol.build_subscribe(channel, &args);
            self.send_raw_json(adhoc.req_unsub).await
        }
    }

    /// 封装心跳与消息接收的 select! 逻辑
    async fn run_ws_with_heartbeat(
        mut read: impl Stream<Item = Result<Message, WsError>> + Unpin,
        tx_out: Sender<serde_json::Value>,
        tx_in: Sender<Message>,
        last_ping_time: Arc<Mutex<Instant>>,
        heartbeat_interval: Duration,
        ping_text: Option<String>,
    ) {
        let mut waiting_pong = false;
        let mut ping_sent_time: Option<Instant> = None;
        loop {
            tokio::select! {
                msg_result = read.next() => {
                    if let Some(res) = msg_result {
                        if let Err(_) = Self::handle_ws_message(
                            res, &tx_out, &tx_in, &last_ping_time, &mut waiting_pong, &mut ping_sent_time
                        ).await {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                _ = sleep(heartbeat_interval) => {
                    if !waiting_pong {
                        let ping_frame = match &ping_text {
                            Some(s) => Message::Text(Utf8Bytes::from(s)),
                            None => Message::Ping(Bytes::from_static(b"")),
                        };
                        if let Err(e) = tx_in.send(ping_frame).await {
                            error!("发送Ping消息失败: {}", e);
                            break;
                        }
                        debug!("已发送Ping消息");
                        waiting_pong = true;
                        ping_sent_time = Some(Instant::now());
                    } else {
                        error!("心跳超时，未收到pong，准备重连...");
                        break;
                    }
                }
            }
        }
    }

    /// 处理单条 WebSocket 消息
    async fn handle_ws_message(
        res: Result<Message, WsError>,
        tx_out: &Sender<serde_json::Value>,
        tx_in: &Sender<Message>,
        last_ping_time: &Arc<Mutex<Instant>>,
        waiting_pong: &mut bool,
        ping_sent_time: &mut Option<Instant>,
    ) -> Result<(), ()> {
        match res {
            Ok(msg) => match &msg {
                Message::Text(text) => {
                    debug!("收到WebSocket消息: {}", text);
                    match serde_json::from_str::<serde_json::Value>(text) {
                        Ok(json_value) => {
                            if let Err(e) = tx_out.send(json_value).await {
                                error!("发送接收的消息到通道错误: {}", e);
                                return Err(());
                            }
                        }
                        Err(e) => {
                            error!("解析WebSocket消息错误: {}", e);
                        }
                    }
                }
                Message::Ping(data) => {
                    debug!("收到Ping消息");
                    *last_ping_time.lock().unwrap() = Instant::now();
                    if let Err(e) = tx_in.send(Message::Pong(data.clone())).await {
                        error!("发送Pong响应错误: {}", e);
                    }
                }
                Message::Pong(_) => {
                    debug!("收到Pong响应");
                    *waiting_pong = false;
                    *ping_sent_time = None;
                }
                _ => {}
            },
            Err(e) => {
                error!("WebSocket接收错误: {}", e);
                return Err(());
            }
        }
        Ok(())
    }

    /// 发送原始 JSON 消息
    async fn send_raw_json(&self, message: serde_json::Value) -> Result<(), Error> {
        if let Some(tx) = &self.tx {
            let message_str = serde_json::to_string(&message).map_err(|e| Error::JsonError(e))?;
            debug!("发送WebSocket消息: {}", message_str);
            tx.send(Message::Text(Utf8Bytes::from(message_str)))
                .await
                .map_err(|e| Error::WebSocketError(format!("发送WebSocket消息失败: {}", e)))?;
            Ok(())
        } else {
            Err(Error::WebSocketError("WebSocket未连接".to_string()))
        }
    }

    /// 启动重连任务
    fn start_reconnect_task(&mut self) {
        if self.reconnect_task.is_some() {
            return;
        }
        let tx = self.tx.clone();
        let last_ping_time = self.last_ping_time.clone();
        let mut client = self.clone();
        self.reconnect_task = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                let should_reconnect = {
                    if let Ok(time) = last_ping_time.lock() {
                        let elapsed = time.elapsed();
                        elapsed > Duration::from_secs(30)
                    } else {
                        false
                    }
                };
                if should_reconnect {
                    warn!("WebSocket连接已超过30秒未活动，尝试重连");
                    if let Some(tx) = &tx {
                        let _ = tx.send(Message::Close(None)).await;
                    }
                    match client.connect().await {
                        Ok(_) => {
                            info!("WebSocket重连成功");
                        }
                        Err(e) => {
                            error!("WebSocket重连失败: {}", e);
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }));
    }
}

impl<P> Clone for WebsocketClient<P>
where
    P: WsProtocol + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            is_private: self.is_private,
            credentials: self.credentials.clone(),
            is_simulated: self.is_simulated.clone(),
            subscriptions: self.subscriptions.clone(),
            tx: self.tx.clone(),
            rx: None,
            connection_task: None,
            reconnect_task: None,
            last_ping_time: self.last_ping_time.clone(),
            protocol: self.protocol.clone(),
        }
    }
}

impl<P> Drop for WebsocketClient<P>
where
    P: WsProtocol + Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if let Some(handle) = self.connection_task.take() {
            handle.abort();
        }
        if let Some(handle) = self.reconnect_task.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::time::sleep;
    #[tokio::test]
    async fn test_subscribe() {
        let args = Args::new().with_inst_id("BTC-USDT".to_string());
        let mut client = crate::OkxWebsocketClient::new_public();
        let mut rx = client.connect().await.unwrap();
        client.subscribe(ChannelType::Tickers, args).await.unwrap();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                println!("收到公共频道消息: {:?}", msg);
            }
        });
        sleep(Duration::from_secs(100)).await;
    }
    #[tokio::test]
    async fn test_unsubscribe() {
        dotenv::dotenv().ok();
        let api_key = env::var("OKX_API_KEY").expect("OKX_API_KEY 未设置");
        let api_secret = env::var("OKX_API_SECRET").expect("OKX_API_SECRET 未设置");
        let passphrase = env::var("OKX_PASSPHRASE").expect("OKX_PASSPHRASE 未设置");
        let mut client = crate::OkxWebsocketClient::new_private(Credentials::new(
            api_key, api_secret, passphrase, "0",
        ));
        let mut rx_private = client.connect().await.unwrap();
        let args = Args::new()
            .with_inst_id("BTC-USDT".to_string())
            .with_param("period".to_string(), "1D".to_string());
        client
            .subscribe(ChannelType::Candle("1D".to_string()), args)
            .await
            .unwrap();
        tokio::spawn(async move {
            while let Some(msg) = rx_private.recv().await {
                println!("收到私有频道消息: {:?}", msg);
            }
        });
        sleep(Duration::from_secs(100)).await;
    }
}

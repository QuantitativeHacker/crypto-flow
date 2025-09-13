/// 账户
/// 每个账户都有一个会话管理器，用于管理与Binance的WebSocket连接
/// 每个账户都有一个用户数据流状态，用于记录当前订阅的用户数据流
///  
use serde_json::Value;
use tracing::{info, warn};

use crate::{
    event_handlers::UserDataEventHandler,
    model::{
        session::SessionLogonResponse,
        user_data::{
            SessionSubscriptionsResponse, UserDataEvent, UserDataStreamState,
            UserDataSubscribeResponse, UserDataUnsubscribeResponse,
        },
        Event, EventMessage,
    },
    session_manager::SessionManager,
};
use websocket::Credentials;

/// 用户数据流管理器
/// 专门负责用户数据流的订阅、取消订阅和事件处理
/// 每个Account有一个SessionManager
pub struct Account<T: UserDataEventHandler> {
    /// 会话管理器
    session_manager: SessionManager,
    /// 用户数据流状态
    user_data_state: UserDataStreamState,
    /// 事件处理器
    event_handler: T,
    /// 消息接收通道
    rx: tokio::sync::mpsc::Receiver<Value>,
    /// 是否已断开连接
    disconnected: bool,
}

impl<T: UserDataEventHandler> Account<T> {
    /// 创建新的用户数据流管理器
    pub async fn new(credentials: &Credentials, event_handler: T) -> Self {
        let mut session_manager = SessionManager::new();
        let rx = session_manager
            .login(&credentials)
            .await
            .expect("Account账户登录失败");
        Self {
            session_manager,
            user_data_state: UserDataStreamState::default(),
            event_handler,
            rx,
            disconnected: false,
        }
    }

    /// 检查是否已断开连接
    pub fn disconnected(&self) -> bool {
        self.disconnected
    }

    /// 订阅用户数据流
    pub async fn subscribe_user_data(&mut self) -> anyhow::Result<u32> {
        if !self.session_manager.is_authenticated() {
            return Err(anyhow::anyhow!("必须先认证才能订阅用户数据流"));
        }

        if !self.user_data_state.can_create_subscription() {
            return Err(anyhow::anyhow!("已达到订阅限制"));
        }

        if let Some(ws_client) = self.session_manager.get_client() {
            let params = serde_json::Value::Object(serde_json::Map::new());
            let id = next_request_id();

            ws_client
                .wsapi_call("userDataStream.subscribe", params, id)
                .await
                .map_err(|e| anyhow::anyhow!("发送用户数据流订阅请求失败: {}", e))?;

            info!("已发送用户数据流订阅请求: {}", id);

            // 返回占位符 ID，实际 ID 将在响应中获得
            Ok(0)
        } else {
            Err(anyhow::anyhow!("WebSocket 客户端未初始化"))
        }
    }

    /// 取消订阅用户数据流
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - 要取消订阅的用户数据流 ID
    /// 如果没有参数，将取消所有订阅
    pub async fn unsubscribe_user_data(
        &mut self,
        subscription_id: Option<u32>,
    ) -> anyhow::Result<()> {
        if !self.session_manager.is_authenticated() {
            return Err(anyhow::anyhow!("必须先认证才能取消订阅"));
        }

        if let Some(ws_client) = self.session_manager.get_client() {
            let params = if let Some(id) = subscription_id {
                serde_json::json!({ "subscriptionId": id })
            } else {
                serde_json::Value::Object(serde_json::Map::new())
            };

            let request_id = next_request_id();

            ws_client
                .wsapi_call("userDataStream.unsubscribe", params, request_id)
                .await
                .map_err(|e| anyhow::anyhow!("发送取消订阅请求失败: {}", e))?;

            info!(
                "已发送取消订阅请求: {}, subscription_id: {:?}",
                request_id, subscription_id
            );

            // 如果指定了订阅 ID，从本地状态中移除
            if let Some(id) = subscription_id {
                self.user_data_state.remove_subscription(id);
            } else {
                self.user_data_state.clear_all_subscriptions();
            }
        } else {
            return Err(anyhow::anyhow!("WebSocket 客户端未初始化"));
        }

        Ok(())
    }

    /// 获取当前订阅列表
    pub async fn get_subscriptions(&self) -> anyhow::Result<()> {
        if let Some(ws_client) = self.session_manager.get_client() {
            let params = serde_json::Value::Object(serde_json::Map::new());
            let id = next_request_id();

            ws_client
                .wsapi_call("session.subscriptions", params, id)
                .await
                .map_err(|e| anyhow::anyhow!("获取订阅列表失败: {}", e))?;

            info!("已请求订阅列表: {}", id);
        } else {
            return Err(anyhow::anyhow!("WebSocket 客户端未初始化"));
        }

        Ok(())
    }

    /// 获取用户数据流状态
    pub fn get_stream_state(&self) -> &UserDataStreamState {
        &self.user_data_state
    }

    /// 处理消息（主要的事件循环）
    /// Account有三种信息要处理：
    /// 1. 用户数据事件
    /// 2. 登录响应
    /// 3. 订阅用户数据响应
    /// 4. 取消订阅响应
    /// 5. 查询当前订阅列表响应
    pub async fn process(&mut self) -> anyhow::Result<Option<String>> {
        // info!("account process, try to recv");
        match self.rx.try_recv() {
            Ok(inner) => {
                // 1) 先尝试解析为 普通事件 格式 { subscriptionId, event }
                if let Ok(event_message) = serde_json::from_value::<EventMessage>(inner.clone()) {
                    if let Event::UserDataEvent(event) = event_message.event {
                        self.handle_user_data_event(&event).await?;
                    } else {
                        info!("Account收到Event:非数据推送: {:?}", event_message);
                    }
                }

                // 2) 登录响应
                if let Ok(response) = serde_json::from_value::<SessionLogonResponse>(inner.clone())
                {
                    // 收到登录响应，更新会话状态
                    self.handle_login_response(&response).await;
                    // 必须认证之后才能订阅
                    self.subscribe_user_data().await?;
                    return Ok(None);
                }

                // 3) 订阅用户数据响应 { id, status, result: { subscriptionId }, rateLimits }
                if let Ok(resp) = serde_json::from_value::<UserDataSubscribeResponse>(inner.clone())
                {
                    self.handle_user_data_subscribe_response(&resp).await;
                    return Ok(None);
                }

                // 4) 取消订阅响应
                if let Ok(resp) =
                    serde_json::from_value::<UserDataUnsubscribeResponse>(inner.clone())
                {
                    if resp.status == 200 {
                        info!("用户数据取消订阅响应成功 (id={})", resp.id);
                        // 注：此响应没有携带 subscriptionId，实际移除在收到服务端推送的 list 结果时统一对齐
                    } else {
                        warn!("取消订阅失败: status={}, err={:?}", resp.status, resp.error);
                    }
                    return Ok(None);
                }

                // 5) 查询当前订阅列表响应
                if let Ok(resp) =
                    serde_json::from_value::<SessionSubscriptionsResponse>(inner.clone())
                {
                    self.handle_session_subscriptions_response(&resp);
                    return Ok(None);
                }

                // 6) 其他未知消息，丢弃，不再上抛，避免上层解析为 EventMessage 报错
                warn!("收到未识别的用户数据消息格式: {:?}", inner);
                return Ok(None);
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                // 没有消息，正常情况
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                warn!("消息通道已断开");
                self.disconnected = true;
            }
        }

        Ok(None)
    }

    /// 处理用户数据事件
    async fn handle_user_data_event(&self, event: &UserDataEvent) -> anyhow::Result<()> {
        info!("Account收到Event:用户数据: {:?}", event);
        let handler = &self.event_handler;
        match event {
            UserDataEvent::ExecutionReport(report) => {
                handler.on_execution_report(report);
            }
            UserDataEvent::BalanceUpdate(balance) => {
                handler.on_balance_update(balance);
            }
            UserDataEvent::OutboundAccountPosition(position) => {
                handler.on_account_position(position);
            }
            UserDataEvent::UserLiabilityChange(liability) => {
                handler.on_user_liability_change(liability);
            }
            UserDataEvent::MarginLevelStatusChange(margin) => {
                handler.on_margin_level_status_change(margin);
            }
            UserDataEvent::ListenStatus(status) => {
                handler.on_listen_status(status);
            }
            UserDataEvent::SpotExpired(expired) => {
                handler.on_spot_expired(expired);
            }
        }
        Ok(())
    }

    async fn handle_login_response(&mut self, response: &SessionLogonResponse) {
        info!("Account收到登录响应: {:?}", response);
        self.session_manager.handle_login_response(response);
    }

    /// 处理用户数据订阅响应
    async fn handle_user_data_subscribe_response(&mut self, resp: &UserDataSubscribeResponse) {
        info!("用户数据订阅响应: {:?}", resp);
        if resp.status == 200 {
            if let Some(result) = &resp.result {
                let _ = self
                    .user_data_state
                    .add_subscription(result.subscription_id);
                info!(
                    "用户数据订阅成功，subscriptionId={} (id={})",
                    result.subscription_id, resp.id
                );
            } else {
                warn!("订阅响应缺少 result 字段: {:?}", resp);
            }
        } else {
            warn!("订阅失败: status={}, err={:?}", resp.status, resp.error);
        }
    }

    /// 处理会话订阅列表响应
    fn handle_session_subscriptions_response(&mut self, resp: &SessionSubscriptionsResponse) {
        if resp.status == 200 {
            if let Some(list) = &resp.result {
                // 用服务端列表对齐本地状态
                self.user_data_state.clear_all_subscriptions();
                for s in list {
                    let _ = self.user_data_state.add_subscription(s.subscription_id);
                }
                info!(
                    "已同步订阅列表，共{}条",
                    self.user_data_state.active_count()
                );
            }
        } else {
            warn!(
                "获取订阅列表失败: status={}, err={:?}",
                resp.status, resp.error
            );
        }
    }

    /// 获取活跃订阅数量
    pub fn get_active_subscription_count(&self) -> usize {
        self.user_data_state.active_count()
    }

    /// 获取订阅限制信息
    pub fn get_subscription_limits(&self) -> (usize, usize, u32, u32) {
        (
            self.user_data_state.active_count(),
            self.user_data_state.max_concurrent_subscriptions as usize,
            self.user_data_state.lifetime_subscription_count,
            self.user_data_state.max_lifetime_subscriptions,
        )
    }
}

/// 生成递增的请求 ID
fn next_request_id() -> i64 {
    static mut COUNTER: i64 = 1;
    unsafe {
        let result = COUNTER;
        COUNTER += 1;
        result
    }
}

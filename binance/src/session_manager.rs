use crate::model::session::{
    SessionLogonResponse, SessionLogonResult, SessionLogoutResponse, SessionStatusResponse,
};
use serde_json::Value;
use tracing::{error, info, warn};
use websocket::{BinanceWsApiWebsocketClient, Credentials};

/// WebSocket 会话状态
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    /// 未连接
    Disconnected,
    /// 已连接但未认证
    Connected,
    /// 已认证
    Authenticated {
        api_key: String,
        authorized_since: i64,
        server_time: i64,
        user_data_stream: bool,
    },
    /// 认证失败
    AuthenticationFailed(String),
}

/// WebSocket 会话管理器
/// 负责管理 WebSocket 连接、认证状态和会话生命周期
pub struct SessionManager {
    /// WebSocket 客户端
    ws_client: Option<BinanceWsApiWebsocketClient>,
    /// 当前会话状态
    state: SessionState,
    /// 认证凭据
    credentials: Option<Credentials>,
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new() -> Self {
        Self {
            ws_client: None,
            state: SessionState::Disconnected,
            credentials: None,
        }
    }

    /// 连接到 WebSocket 服务器
    pub async fn connect(&mut self) -> anyhow::Result<tokio::sync::mpsc::Receiver<Value>> {
        info!("正在连接到 WebSocket 服务器");

        let mut ws_client = BinanceWsApiWebsocketClient::new_public("session_manager");

        // 连接并获取接收通道
        let rx = ws_client
            .connect()
            .await
            .map_err(|e| anyhow::anyhow!("WebSocket 连接失败: {}", e))?;

        self.ws_client = Some(ws_client);
        self.state = SessionState::Connected;

        info!("WebSocket 连接成功");
        Ok(rx)
    }

    /// 使用凭据登录
    pub async fn login(
        &mut self,
        credentials: &Credentials,
    ) -> anyhow::Result<tokio::sync::mpsc::Receiver<Value>> {
        info!("正在进行 WebSocket API 认证...");
        let mut ws_client =
            BinanceWsApiWebsocketClient::new_private("session_manager", credentials.clone());
        let rx = ws_client
            .connect()
            .await
            .map_err(|e| anyhow::anyhow!("私有连接失败: {}", e))?;
        self.ws_client = Some(ws_client);
        self.credentials = Some(credentials.clone());

        // 这里应该监听登录响应并更新状态
        // 实际实现中需要处理异步响应
        info!("登录请求已发送，等待服务器响应...");

        Ok(rx)
    }

    /// 登出
    pub async fn logout(&mut self) -> anyhow::Result<()> {
        if !self.is_authenticated() {
            warn!("当前未认证，无需登出");
            return Ok(());
        }

        if let Some(ws_client) = &self.ws_client {
            // 发送登出请求
            ws_client
                .wsapi_call(
                    "session.logout",
                    serde_json::Value::Object(serde_json::Map::new()),
                    999,
                )
                .await
                .map_err(|e| anyhow::anyhow!("发送登出请求失败: {}", e))?;
        }

        self.state = SessionState::Connected;
        info!("已发送登出请求");
        Ok(())
    }

    /// 获取会话状态
    pub async fn get_status(&self) -> anyhow::Result<()> {
        if let Some(ws_client) = &self.ws_client {
            ws_client
                .wsapi_call(
                    "session.status",
                    serde_json::Value::Object(serde_json::Map::new()),
                    998,
                )
                .await
                .map_err(|e| anyhow::anyhow!("获取会话状态失败: {}", e))?;
            info!("已请求会话状态");
        } else {
            return Err(anyhow::anyhow!("WebSocket 客户端未初始化"));
        }
        Ok(())
    }

    /// 重新连接
    pub async fn reconnect(&mut self) -> anyhow::Result<()> {
        info!("正在重新连接...");

        let credentials = self.credentials.clone();

        // 重置状态
        self.state = SessionState::Disconnected;
        self.ws_client = None;

        // 重新连接
        self.connect().await?;

        // 如果有凭据，重新登录
        if let Some(cred) = credentials {
            self.login(&cred).await?;
        }

        info!("重新连接完成");
        Ok(())
    }

    /// 检查是否已认证
    pub fn is_authenticated(&self) -> bool {
        matches!(self.state, SessionState::Authenticated { .. })
    }

    /// 检查是否已连接
    pub fn is_connected(&self) -> bool {
        !matches!(self.state, SessionState::Disconnected)
    }

    /// 获取当前会话状态
    pub fn get_state(&self) -> &SessionState {
        &self.state
    }

    /// 获取 WebSocket 客户端引用（用于其他组件）
    pub fn get_client(&self) -> Option<&BinanceWsApiWebsocketClient> {
        self.ws_client.as_ref()
    }

    /// 获取可变的 WebSocket 客户端引用
    pub fn get_client_mut(&mut self) -> Option<&mut BinanceWsApiWebsocketClient> {
        self.ws_client.as_mut()
    }

    /// 处理登录响应（由外部调用）
    pub fn handle_login_result(&mut self, result: &SessionLogonResult) {
        self.state = SessionState::Authenticated {
            api_key: result.api_key.clone(),
            authorized_since: result.authorized_since,
            server_time: result.server_time,
            user_data_stream: result.user_data_stream,
        };
        info!(
            "WebSocket API 认证成功: api_key={}, user_data_stream={}",
            result.api_key, result.user_data_stream
        );
    }

    /// 处理登录完整响应（状态码 + 结果/错误）
    pub fn handle_login_response(&mut self, response: &SessionLogonResponse) {
        if response.status == 200 {
            if let Some(result) = &response.result {
                self.handle_login_result(result);
            } else {
                warn!("登录响应状态为200但缺少result字段");
            }
        } else {
            let error_msg = response
                .error
                .as_ref()
                .map(|e| format!("code={}, msg={}", e.code, e.msg))
                .unwrap_or_else(|| "未知错误".to_string());
            self.state = SessionState::AuthenticationFailed(error_msg.clone());
            error!("WebSocket API 认证失败: {}", error_msg);
        }
    }

    /// 处理登出响应（由外部调用）
    pub fn handle_logout_response(&mut self, response: &SessionLogoutResponse) {
        if response.status == 200 {
            self.state = SessionState::Connected;
            info!("WebSocket API 登出成功");
        } else {
            let error_msg = response
                .error
                .as_ref()
                .map(|e| format!("code={}, msg={}", e.code, e.msg))
                .unwrap_or_else(|| "未知错误".to_string());
            error!("WebSocket API 登出失败: {}", error_msg);
        }
    }

    /// 处理状态响应（由外部调用）
    pub fn handle_status_response(&mut self, response: &SessionStatusResponse) {
        if response.status == 200 {
            if let Some(result) = &response.result {
                info!("会话状态: server_time={}", result.server_time);
            }
        } else {
            let error_msg = response
                .error
                .as_ref()
                .map(|e| format!("code={}, msg={}", e.code, e.msg))
                .unwrap_or_else(|| "未知错误".to_string());
            error!("获取会话状态失败: {}", error_msg);
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

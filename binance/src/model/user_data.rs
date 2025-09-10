use crate::model::ExecutionReport;
use serde::{Deserialize, Serialize};

/// 用户数据流订阅结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionResult {
    /// 订阅 ID
    #[serde(rename = "subscriptionId")]
    pub subscription_id: u32,
}

/// 用户数据流订阅响应
pub type UserDataSubscribeResponse = crate::model::session::WsApiResponse<SubscriptionResult>;

/// 用户数据流取消订阅响应（空结果）
pub type UserDataUnsubscribeResponse = crate::model::session::WsApiResponse<serde_json::Value>;

/// 会话订阅列表结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionSubscription {
    /// 订阅 ID
    #[serde(rename = "subscriptionId")]
    pub subscription_id: u32,
}

/// 会话订阅列表响应
pub type SessionSubscriptionsResponse =
    crate::model::session::WsApiResponse<Vec<SessionSubscription>>;

/// Spot 账户位置信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpotPosition {
    /// 资产名称
    pub a: String,
    /// 可用余额
    pub f: String,
    /// 冻结余额
    pub l: String,
}

/// 账户位置更新事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutboundAccountPosition {
    // 移除 e 字段以兼容内部标记枚举 #[serde(tag = "e")]
    #[serde(rename = "E")]
    pub E: i64,
    pub u: i64,
    #[serde(rename = "B")]
    pub B: Vec<SpotPosition>,
}

/// 余额更新事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceUpdate {
    // 移除 e 字段以兼容内部标记枚举 #[serde(tag = "e")]
    #[serde(rename = "E")]
    pub E: i64,
    pub a: String,
    pub d: String,
    #[serde(rename = "T")]
    pub T: i64,
}

/// Spot 监听密钥过期事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpotExpired {
    // 移除 e 字段以兼容内部标记枚举 #[serde(tag = "e")]
    #[serde(rename = "E")]
    pub E: i64,
    #[serde(rename = "listenKey")]
    pub listenKey: String,
}

/// 用户责任变化事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserLiabilityChange {
    // 移除 e 字段以兼容内部标记枚举 #[serde(tag = "e")]
    #[serde(rename = "E")]
    pub E: i64,
    pub a: String,
    pub t: String,
    pub p: String,
    pub i: String,
}

/// 保证金水平状态变化事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarginLevelStatusChange {
    // 移除 e 字段以兼容内部标记枚举 #[serde(tag = "e")]
    #[serde(rename = "E")]
    pub E: i64,
    pub l: String,
    pub s: String,
}

/// OCO 订单详情
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OCODetails {
    /// 交易对
    pub s: String,
    /// 订单 ID
    pub i: i64,
    /// 客户端订单 ID
    pub c: String,
}

/// 监听状态事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListenStatus {
    // 移除 e 字段以兼容内部标记枚举 #[serde(tag = "e")]
    #[serde(rename = "E")]
    pub E: i64,
    pub s: String,
    pub g: i64,
    pub o: String,
    pub l: String,
    #[serde(rename = "L")]
    pub L: String,
    pub r: String,
}


/// 用户数据事件类型
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "e")]
pub enum UserDataEvent {
    /// 订单执行报告
    #[serde(rename = "executionReport")]
    ExecutionReport(ExecutionReport),

    /// 账户余额更新
    #[serde(rename = "balanceUpdate")]
    BalanceUpdate(BalanceUpdate),

    /// 账户位置更新
    #[serde(rename = "outboundAccountPosition")]
    OutboundAccountPosition(OutboundAccountPosition),

    /// 用户责任变化
    #[serde(rename = "userLiabilityChange")]
    UserLiabilityChange(UserLiabilityChange),

    /// 保证金水平状态变化
    #[serde(rename = "marginLevelStatusChange")]
    MarginLevelStatusChange(MarginLevelStatusChange),

    /// 监听状态
    #[serde(rename = "listStatus")]
    ListenStatus(ListenStatus),

    /// Spot 过期事件
    #[serde(rename = "listenKeyExpired")]
    SpotExpired(SpotExpired),
}

/// 用户数据流管理器状态
#[derive(Debug, Clone)]
pub struct UserDataStreamState {
    /// 活跃订阅列表
    pub active_subscriptions: Vec<u32>,
    /// 最大并发订阅数（1000）
    pub max_concurrent_subscriptions: u32,
    /// 生命周期内最大订阅数（65535）
    pub max_lifetime_subscriptions: u32,
    /// 当前生命周期订阅计数
    pub lifetime_subscription_count: u32,
}

impl Default for UserDataStreamState {
    fn default() -> Self {
        Self {
            active_subscriptions: Vec::new(),
            max_concurrent_subscriptions: 1000,
            max_lifetime_subscriptions: 65535,
            lifetime_subscription_count: 0,
        }
    }
}

impl UserDataStreamState {
    /// 检查是否可以创建新订阅
    pub fn can_create_subscription(&self) -> bool {
        self.active_subscriptions.len() < self.max_concurrent_subscriptions as usize
            && self.lifetime_subscription_count < self.max_lifetime_subscriptions
    }

    /// 添加新订阅
    pub fn add_subscription(&mut self, subscription_id: u32) -> Result<(), String> {
        if !self.can_create_subscription() {
            return Err("达到订阅限制".to_string());
        }

        self.active_subscriptions.push(subscription_id);
        self.lifetime_subscription_count += 1;
        Ok(())
    }

    /// 移除订阅
    pub fn remove_subscription(&mut self, subscription_id: u32) -> bool {
        if let Some(pos) = self
            .active_subscriptions
            .iter()
            .position(|&x| x == subscription_id)
        {
            self.active_subscriptions.remove(pos);
            true
        } else {
            false
        }
    }

    /// 清除所有订阅
    pub fn clear_all_subscriptions(&mut self) {
        self.active_subscriptions.clear();
    }

    /// 获取活跃订阅数量
    pub fn active_count(&self) -> usize {
        self.active_subscriptions.len()
    }
}

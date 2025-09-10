use crate::model::{
    ExecutionReport,
    user_data::{
        BalanceUpdate, OutboundAccountPosition, SpotExpired, 
        UserLiabilityChange, MarginLevelStatusChange, ListenStatus
    },
    bookticker::BinanceBookTicker,
    depth::{BinanceSpotDepth, BinanceFutureDepth},
    kline::BinanceKline,
};

/// 用户数据事件处理器接口
/// 实现此 trait 来处理各种用户数据事件
pub trait UserDataEventHandler: Send + Sync {
    /// 处理订单执行报告
    fn on_execution_report(&self, report: &ExecutionReport) {
        // 默认实现：记录日志
        tracing::info!("收到订单执行报告: symbol={}, side={:?}, status={:?}, order_id={}", 
            report.s, report.S, report.X, report.i);
    }

    /// 处理余额更新事件
    fn on_balance_update(&self, update: &BalanceUpdate) {
        tracing::info!("收到余额更新: asset={}, delta={}, time={}", 
            update.a, update.d, update.T);
    }

    /// 处理账户位置更新事件
    fn on_account_position(&self, position: &OutboundAccountPosition) {
        tracing::info!("收到账户位置更新: event_time={}, balances_count={}", 
            position.E, position.B.len());
    }

    /// 处理用户责任变化事件
    fn on_user_liability_change(&self, liability: &UserLiabilityChange) {
        tracing::info!("收到用户责任变化: asset={}, type={}, principal={}", 
            liability.a, liability.t, liability.p);
    }

    /// 处理保证金水平状态变化事件
    fn on_margin_level_status_change(&self, margin: &MarginLevelStatusChange) {
        tracing::info!("收到保证金水平状态变化: level={}, status={}", 
            margin.l, margin.s);
    }

    /// 处理监听状态事件
    fn on_listen_status(&self, status: &ListenStatus) {
        tracing::info!("收到监听状态: symbol={}, order_list_id={}", 
            status.s, status.g);
    }

    /// 处理 Spot 过期事件
    fn on_spot_expired(&self, expired: &SpotExpired) {
        tracing::warn!("Spot 监听密钥过期: listen_key={}", expired.listenKey);
    }

    /// 处理未知或自定义事件
    fn on_unknown_event(&self, event_type: &str, data: &serde_json::Value) {
        tracing::warn!("收到未知用户数据事件: type={}, data={:?}", event_type, data);
    }
}

/// 市场数据事件处理器接口
/// 实现此 trait 来处理各种市场数据事件
pub trait MarketEventHandler: Send + Sync {
    /// 处理 24hr 价格变动统计
    fn on_ticker(&self, ticker: &BinanceBookTicker) {
        tracing::info!("收到价格统计: symbol={}, bid_price={}, ask_price={}", 
            ticker.data.s, ticker.data.b, ticker.data.a);
    }

    /// 处理现货深度信息
    fn on_spot_depth(&self, depth: &BinanceSpotDepth) {
        tracing::info!("收到现货深度: stream={}, bids={}, asks={}", 
            depth.stream, depth.data.bids.len(), depth.data.asks.len());
    }

    /// 处理期货深度信息
    fn on_future_depth(&self, depth: &BinanceFutureDepth) {
        tracing::info!("收到期货深度: symbol={}, bids={}, asks={}", 
            depth.data.s, depth.data.b.len(), depth.data.a.len());
    }

    /// 处理 K线数据
    fn on_kline(&self, kline: &BinanceKline) {
        tracing::info!("收到K线数据: symbol={}, interval={}, open={}, close={}", 
            kline.data.s, kline.data.k.i, kline.data.k.o, kline.data.k.c);
    }

    /// 处理未知或自定义市场事件
    fn on_unknown_market_event(&self, event_type: &str, data: &serde_json::Value) {
        tracing::warn!("收到未知市场数据事件: type={}, data={:?}", event_type, data);
    }
}

/// 默认的用户数据事件处理器
/// 提供基本的日志记录功能
pub struct DefaultUserDataHandler;

impl UserDataEventHandler for DefaultUserDataHandler {
    // 使用默认实现，所有事件都会记录到日志
}

/// 默认的市场数据事件处理器
/// 提供基本的日志记录功能
pub struct DefaultMarketDataHandler;

impl MarketEventHandler for DefaultMarketDataHandler {
    // 使用默认实现，所有事件都会记录到日志
}

/// 自定义用户数据事件处理器示例
/// 展示如何实现自定义的事件处理逻辑
pub struct CustomUserDataHandler {
    pub name: String,
}

impl UserDataEventHandler for CustomUserDataHandler {
    fn on_execution_report(&self, report: &ExecutionReport) {
        // 自定义处理逻辑
        println!("[{}] 订单执行: {} {:?} {} @ {}", 
            self.name, report.s, report.S, report.q, report.p);
        
        // 可以在这里添加更复杂的业务逻辑
        // 例如：更新本地订单状态、发送通知、记录到数据库等
    }

    fn on_balance_update(&self, update: &BalanceUpdate) {
        // 自定义余额处理逻辑
        println!("[{}] 余额变化: {} {}", 
            self.name, update.a, update.d);
        
        // 可以在这里添加：
        // - 更新本地余额缓存
        // - 触发风险管理检查
        // - 发送余额变化通知
    }

    fn on_account_position(&self, position: &OutboundAccountPosition) {
        // 自定义账户位置处理逻辑
        println!("[{}] 账户更新: {} 个资产", 
            self.name, position.B.len());
        
        // 可以在这里添加：
        // - 更新持仓信息
        // - 计算总资产价值
        // - 风险评估
    }
}

/// 自定义市场数据事件处理器示例
pub struct CustomMarketDataHandler {
    pub strategy_name: String,
}

impl MarketEventHandler for CustomMarketDataHandler {
    fn on_ticker(&self, ticker: &BinanceBookTicker) {
        // 自定义价格处理逻辑
        println!("[{}] 价格更新: {} bid={} ask={}", 
            self.strategy_name, ticker.data.s, ticker.data.b, ticker.data.a);
        
        // 可以在这里添加：
        // - 价格分析
        // - 交易信号生成
        // - 套利机会检测
    }

    fn on_spot_depth(&self, depth: &BinanceSpotDepth) {
        // 自定义深度处理逻辑
        if let (Some(best_bid), Some(best_ask)) = (depth.data.bids.first(), depth.data.asks.first()) {
            println!("[{}] 深度更新: {} 最佳买价={} 最佳卖价={}", 
                self.strategy_name, depth.stream, best_bid.price, best_ask.price);
        }
        
        // 可以在这里添加：
        // - 流动性分析
        // - 订单簿不平衡检测
        // - 市场微观结构分析
    }

    fn on_kline(&self, kline: &BinanceKline) {
        // 自定义K线处理逻辑
        println!("[{}] K线更新: {} {} OHLC=({},{},{},{})", 
            self.strategy_name, kline.data.s, kline.data.k.i, 
            kline.data.k.o, kline.data.k.h, kline.data.k.l, kline.data.k.c);
        
        // 可以在这里添加：
        // - 技术指标计算
        // - 趋势分析
        // - 交易信号生成
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_handlers() {
        let user_handler = DefaultUserDataHandler;
        let market_handler = DefaultMarketDataHandler;
        
        // 测试默认处理器不会崩溃
        user_handler.on_unknown_event("test", &json!({"test": "data"}));
        market_handler.on_unknown_market_event("test", &json!({"test": "data"}));
    }

    #[test]
    fn test_custom_handlers() {
        let user_handler = CustomUserDataHandler {
            name: "TestStrategy".to_string(),
        };
        
        let market_handler = CustomMarketDataHandler {
            strategy_name: "TestStrategy".to_string(),
        };
        
        // 测试自定义处理器
        user_handler.on_unknown_event("test", &json!({"test": "data"}));
        market_handler.on_unknown_market_event("test", &json!({"test": "data"}));
    }
}
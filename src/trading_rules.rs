//! 统一的交易规则接口
//!
//! 这个模块定义了一个通用的交易规则 trait，用于抽象不同交易所的交易规则差异。
//! 支持币安、OKX 等多个交易所的统一接口。

use std::fmt::Debug;

/// 统一的交易规则接口
///
/// 这个 trait 定义了所有交易所都应该支持的基本交易规则方法。
/// 不同的交易所可以通过实现这个 trait 来提供统一的接口。
pub trait TradingRules: Debug + Clone {
    /// 获取交易对符号
    fn symbol(&self) -> &String;

    /// 获取最小价格
    /// 返回该交易对允许的最小价格
    fn min_price(&self) -> f64;

    /// 获取最大价格
    /// 返回该交易对允许的最大价格
    fn max_price(&self) -> f64;

    /// 获取价格步长
    /// 返回价格的最小变动单位（tick size）
    fn tick_size(&self) -> f64;

    /// 获取最小数量
    /// 返回该交易对允许的最小订单数量
    fn min_quantity(&self) -> f64;

    /// 获取最大数量
    /// 返回该交易对允许的最大订单数量
    fn max_quantity(&self) -> f64;

    /// 获取数量步长
    /// 返回数量的最小变动单位（lot size）
    fn lot_size(&self) -> f64;

    /// 获取最小名义价值
    /// 返回订单的最小名义价值（价格 × 数量）
    fn min_notional(&self) -> f64;

    /// 验证价格是否有效
    /// 检查给定价格是否符合交易规则
    fn is_valid_price(&self, price: f64) -> bool {
        let min_price = self.min_price();
        let max_price = self.max_price();
        let tick_size = self.tick_size();

        if price < min_price || price > max_price {
            return false;
        }

        if tick_size > 0.0 {
            let remainder = (price / tick_size) % 1.0;
            // 允许小的浮点误差
            remainder < 1e-8 || remainder > (1.0 - 1e-8)
        } else {
            true
        }
    }

    /// 验证数量是否有效
    /// 检查给定数量是否符合交易规则
    fn is_valid_quantity(&self, quantity: f64) -> bool {
        let min_qty = self.min_quantity();
        let max_qty = self.max_quantity();
        let lot_size = self.lot_size();

        if quantity < min_qty || quantity > max_qty {
            return false;
        }

        if lot_size > 0.0 {
            let remainder = (quantity / lot_size) % 1.0;
            // 允许小的浮点误差
            remainder < 1e-8 || remainder > (1.0 - 1e-8)
        } else {
            true
        }
    }

    /// 验证订单是否有效
    /// 检查给定的价格和数量是否都符合交易规则
    fn is_valid_order(&self, price: f64, quantity: f64) -> bool {
        if !self.is_valid_price(price) || !self.is_valid_quantity(quantity) {
            return false;
        }

        let notional = price * quantity;
        let min_notional = self.min_notional();

        notional >= min_notional
    }

    /// 调整价格到有效值
    /// 将给定价格调整为符合交易规则的最接近值
    fn adjust_price(&self, price: f64) -> f64 {
        let min_price = self.min_price();
        let max_price = self.max_price();
        let tick_size = self.tick_size();

        let clamped_price = price.max(min_price).min(max_price);

        if tick_size > 0.0 {
            (clamped_price / tick_size).round() * tick_size
        } else {
            clamped_price
        }
    }

    /// 调整数量到有效值
    /// 将给定数量调整为符合交易规则的最接近值
    fn adjust_quantity(&self, quantity: f64) -> f64 {
        let min_qty = self.min_quantity();
        let max_qty = self.max_quantity();
        let lot_size = self.lot_size();

        let clamped_qty = quantity.max(min_qty).min(max_qty);

        if lot_size > 0.0 {
            (clamped_qty / lot_size).floor() * lot_size
        } else {
            clamped_qty
        }
    }
}

/// 计算订单大小的辅助函数
///
/// 根据交易规则和目标金额计算合适的订单数量
pub fn calculate_order_size<T: TradingRules>(product: &T, target_amount: f64) -> f64 {
    let min_qty = product.min_quantity();
    let lot_size = product.lot_size();

    if lot_size <= 0.0 {
        return min_qty;
    }

    let qty = (target_amount / lot_size).floor() * lot_size;
    qty.max(min_qty)
}

/// 计算最小订单金额的辅助函数
///
/// 根据交易规则计算满足最小名义价值要求的最小订单金额
pub fn calculate_min_order_amount<T: TradingRules>(product: &T, price: f64) -> f64 {
    let min_qty = product.min_quantity();
    let min_notional = product.min_notional();

    let amount_by_qty = price * min_qty;
    let amount_by_notional = min_notional;

    amount_by_qty.max(amount_by_notional)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 创建一个测试用的 TradingRules 实现
    #[derive(Debug, Clone)]
    struct TestProduct {
        symbol: String,
        min_price: f64,
        max_price: f64,
        tick_size: f64,
        min_quantity: f64,
        max_quantity: f64,
        lot_size: f64,
        min_notional: f64,
    }

    impl TradingRules for TestProduct {
        fn symbol(&self) -> &String {
            &self.symbol
        }
        fn min_price(&self) -> f64 {
            self.min_price
        }
        fn max_price(&self) -> f64 {
            self.max_price
        }
        fn tick_size(&self) -> f64 {
            self.tick_size
        }
        fn min_quantity(&self) -> f64 {
            self.min_quantity
        }
        fn max_quantity(&self) -> f64 {
            self.max_quantity
        }
        fn lot_size(&self) -> f64 {
            self.lot_size
        }
        fn min_notional(&self) -> f64 {
            self.min_notional
        }
    }

    #[test]
    fn test_price_validation() {
        let product = TestProduct {
            symbol: "BTCUSDT".to_string(),
            min_price: 0.01,
            max_price: 100000.0,
            tick_size: 0.01,
            min_quantity: 0.001,
            max_quantity: 1000.0,
            lot_size: 0.001,
            min_notional: 10.0,
        };

        assert!(product.is_valid_price(50000.0));
        assert!(!product.is_valid_price(0.005)); // 小于最小价格
        assert!(!product.is_valid_price(150000.0)); // 大于最大价格
        assert!(!product.is_valid_price(50000.005)); // 不符合 tick size
    }

    #[test]
    fn test_quantity_validation() {
        let product = TestProduct {
            symbol: "BTCUSDT".to_string(),
            min_price: 0.01,
            max_price: 100000.0,
            tick_size: 0.01,
            min_quantity: 0.001,
            max_quantity: 1000.0,
            lot_size: 0.001,
            min_notional: 10.0,
        };

        assert!(product.is_valid_quantity(0.001));
        assert!(product.is_valid_quantity(1.0));
        assert!(!product.is_valid_quantity(0.0005)); // 小于最小数量
        assert!(!product.is_valid_quantity(1500.0)); // 大于最大数量
        assert!(!product.is_valid_quantity(0.0015)); // 不符合 lot size
    }

    #[test]
    fn test_order_validation() {
        let product = TestProduct {
            symbol: "BTCUSDT".to_string(),
            min_price: 0.01,
            max_price: 100000.0,
            tick_size: 0.01,
            min_quantity: 0.001,
            max_quantity: 1000.0,
            lot_size: 0.001,
            min_notional: 10.0,
        };

        assert!(product.is_valid_order(50000.0, 0.001)); // 50000 * 0.001 = 50 > 10
        assert!(!product.is_valid_order(5000.0, 0.001)); // 5000 * 0.001 = 5 < 10
    }

    #[test]
    fn test_price_adjustment() {
        let product = TestProduct {
            symbol: "BTCUSDT".to_string(),
            min_price: 0.01,
            max_price: 100000.0,
            tick_size: 0.01,
            min_quantity: 0.001,
            max_quantity: 1000.0,
            lot_size: 0.001,
            min_notional: 10.0,
        };

        assert_eq!(product.adjust_price(50000.005), 50000.01);
        assert_eq!(product.adjust_price(0.005), 0.01);
        assert_eq!(product.adjust_price(150000.0), 100000.0);
    }

    #[test]
    fn test_quantity_adjustment() {
        let product = TestProduct {
            symbol: "BTCUSDT".to_string(),
            min_price: 0.01,
            max_price: 100000.0,
            tick_size: 0.01,
            min_quantity: 0.001,
            max_quantity: 1000.0,
            lot_size: 0.001,
            min_notional: 10.0,
        };

        assert_eq!(product.adjust_quantity(0.0015), 0.001);
        assert_eq!(product.adjust_quantity(0.0005), 0.001);
        assert_eq!(product.adjust_quantity(1500.0), 1000.0);
    }
}

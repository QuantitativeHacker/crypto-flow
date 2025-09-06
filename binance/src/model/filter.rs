use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "filterType")]
#[allow(non_camel_case_types, unused)]
pub enum FilterField {
    // 价格过滤器 - 定义价格规则
    PRICE_FILTER {
        #[serde(rename = "tickSize")]
        tick_size: String, // 价格增减的最小单位
        #[serde(rename = "maxPrice")]
        max_price: String, // 最大价格
        #[serde(rename = "minPrice")]
        min_price: String, // 最小价格
    },

    // 百分比价格过滤器 - 基于平均价格的百分比范围
    PERCENT_PRICE {
        #[serde(rename = "multiplierUp")]
        multiplier_up: String, // 价格上限倍数
        #[serde(rename = "multiplierDown")]
        multiplier_down: String, // 价格下限倍数
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: i32, // 平均价格计算分钟数
    },

    // 按方向百分比价格过滤器 - 买卖方向不同的价格范围
    PERCENT_PRICE_BY_SIDE {
        #[serde(rename = "bidMultiplierUp")]
        bid_multiplier_up: String, // 买单价格上限倍数
        #[serde(rename = "bidMultiplierDown")]
        bid_multiplier_down: String, // 买单价格下限倍数
        #[serde(rename = "askMultiplierUp")]
        ask_multiplier_up: String, // 卖单价格上限倍数
        #[serde(rename = "askMultiplierDown")]
        ask_multiplier_down: String, // 卖单价格下限倍数
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: i32, // 平均价格计算分钟数
    },

    // 数量过滤器 - 定义数量规则
    LOT_SIZE {
        #[serde(rename = "stepSize")]
        step_size: String, // 数量增减的最小单位
        #[serde(rename = "maxQty")]
        max_qty: String, // 最大数量
        #[serde(rename = "minQty")]
        min_qty: String, // 最小数量
    },

    // 市价单数量过滤器 - 市价单的数量规则
    MARKET_LOT_SIZE {
        #[serde(rename = "stepSize")]
        step_size: String, // 数量增减的最小单位
        #[serde(rename = "maxQty")]
        max_qty: String, // 最大数量
        #[serde(rename = "minQty")]
        min_qty: String, // 最小数量
    },

    // 最小名义价值过滤器 - 订单的最小名义价值
    MIN_NOTIONAL {
        #[serde(rename = "minNotional")]
        min_notional: String, // 最小名义价值
        #[serde(rename = "applyToMarket")]
        apply_to_market: bool, // 是否应用于市价单
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: i32, // 平均价格计算分钟数
    },

    // 名义价值过滤器 - 订单的名义价值范围
    NOTIONAL {
        #[serde(rename = "minNotional")]
        min_notional: String, // 最小名义价值
        #[serde(rename = "applyMinToMarket")]
        apply_min_to_market: bool, // 最小名义价值是否应用于市价单
        #[serde(rename = "maxNotional")]
        max_notional: String, // 最大名义价值
        #[serde(rename = "applyMaxToMarket")]
        apply_max_to_market: bool, // 最大名义价值是否应用于市价单
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: i32, // 平均价格计算分钟数
    },

    // 冰山订单部分过滤器 - 冰山订单的最大部分数
    ICEBERG_PARTS {
        limit: i32, // 最大部分数
    },

    // 最大订单数过滤器 - 单个交易对的最大订单数
    MAX_NUM_ORDERS {
        #[serde(rename = "maxNumOrders")]
        max_num_orders: i64, // 最大订单数
    },

    // 最大算法订单数过滤器 - 单个交易对的最大算法订单数
    MAX_NUM_ALGO_ORDERS {
        #[serde(rename = "maxNumAlgoOrders")]
        max_num_algo_orders: i64, // 最大算法订单数
    },

    // 最大冰山订单数过滤器 - 单个交易对的最大冰山订单数
    MAX_NUM_ICEBERG_ORDERS {
        #[serde(rename = "maxNumIcebergOrders")]
        max_num_iceberg_orders: i64, // 最大冰山订单数
    },

    // 最大持仓过滤器 - 基础资产的最大持仓
    MAX_POSITION {
        #[serde(rename = "maxPosition")]
        max_position: String, // 最大持仓
    },

    // 跟踪增量过滤器 - 跟踪止损订单的增量范围
    TRAILING_DELTA {
        #[serde(rename = "minTrailingAboveDelta")]
        min_trailing_above_delta: i32, // 最小跟踪上方增量
        #[serde(rename = "maxTrailingAboveDelta")]
        max_trailing_above_delta: i32, // 最大跟踪上方增量
        #[serde(rename = "minTrailingBelowDelta")]
        min_trailing_below_delta: i32, // 最小跟踪下方增量
        #[serde(rename = "maxTrailingBelowDelta")]
        max_trailing_below_delta: i32, // 最大跟踪下方增量
    },

    // 最大订单修改次数过滤器 - 单个订单的最大修改次数
    MAX_NUM_ORDER_AMENDS {
        #[serde(rename = "maxNumOrderAmends")]
        max_num_order_amends: i64, // 最大修改次数
    },

    // 最大订单列表数过滤器 - 单个交易对的最大订单列表数
    MAX_NUM_ORDER_LISTS {
        #[serde(rename = "maxNumOrderLists")]
        max_num_order_lists: i64, // 最大订单列表数
    },

    // 交易所最大订单数过滤器 - 整个交易所的最大订单数
    EXCHANGE_MAX_NUM_ORDERS {
        #[serde(rename = "maxNumOrders")]
        max_num_orders: i64, // 最大订单数
    },

    // 交易所最大算法订单数过滤器 - 整个交易所的最大算法订单数
    EXCHANGE_MAX_NUM_ALGO_ORDERS {
        #[serde(rename = "maxNumAlgoOrders")]
        max_num_algo_orders: i64, // 最大算法订单数
    },

    // 交易所最大冰山订单数过滤器 - 整个交易所的最大冰山订单数
    EXCHANGE_MAX_NUM_ICEBERG_ORDERS {
        #[serde(rename = "maxNumIcebergOrders")]
        max_num_iceberg_orders: i64, // 最大冰山订单数
    },

    // 交易所最大订单列表数过滤器 - 整个交易所的最大订单列表数
    EXCHANGE_MAX_NUM_ORDER_LISTS {
        #[serde(rename = "maxNumOrderLists")]
        max_num_order_lists: i64, // 最大订单列表数
    },

    // 未知过滤器类型
    #[serde(other)]
    Unknown,
}

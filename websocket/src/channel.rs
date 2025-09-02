use std::collections::HashMap;

// 交易所无关的标的描述
#[derive(Clone, Debug)]
pub enum MarketType {
    Spot,
    UmFuture,
    CmFuture,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Instrument {
    pub symbol: String,     // 统一使用原始字符串，由协议层决定如何规范化
    pub market: MarketType, // 市场类型（可选）
}

#[derive(Clone, Debug)]
pub struct Args {
    pub instrument: Option<Instrument>,
    pub params: HashMap<String, String>,
}

impl Args {
    pub fn new() -> Self {
        Self {
            instrument: None,
            params: HashMap::new(),
        }
    }

    // 兼容旧接口：用 inst_id 设置 symbol，market 默认为 Unknown
    pub fn with_inst_id(mut self, inst: String) -> Self {
        self.instrument = Some(Instrument {
            symbol: inst,
            market: MarketType::Unknown,
        });
        self
    }

    pub fn with_instrument(mut self, instrument: Instrument) -> Self {
        self.instrument = Some(instrument);
        self
    }

    pub fn with_param(mut self, k: String, v: String) -> Self {
        self.params.insert(k, v);
        self
    }

    // 提供便捷访问：获取 symbol 原文
    pub fn symbol(&self) -> Option<&str> {
        self.instrument.as_ref().map(|i| i.symbol.as_str())
    }

    // 提供便捷访问：获取规范化后的小写且移除连字符的 symbol（供 Binance 使用）
    pub fn normalized_symbol(&self) -> Option<String> {
        self.symbol().map(|s| s.replace('-', "").to_lowercase())
    }
}

#[derive(Clone, Debug)]
pub enum ChannelType {
    Tickers,
    Trades,
    Books,
    Depth,
    Candle(String),
}

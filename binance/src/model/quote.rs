use serde::Serialize;

/// 量价信息，表示订单簿中的一个量价对
/// [price, quantity]
#[derive(Debug, Clone, Serialize)]
pub struct BinanceQuote {
    pub price: f64,
    pub quantity: f64,
}

impl<'de> serde::Deserialize<'de> for BinanceQuote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let [price_str, quantity_str]: [String; 2] = serde::Deserialize::deserialize(deserializer)?;

        let price = price_str
            .parse::<f64>()
            .map_err(|_| serde::de::Error::custom("Failed to parse price"))?;
        let quantity = quantity_str
            .parse::<f64>()
            .map_err(|_| serde::de::Error::custom("Failed to parse quantity"))?;

        Ok(BinanceQuote { price, quantity })
    }
}

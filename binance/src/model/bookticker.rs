use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BinanceBookTicker {
    pub stream: String,
    pub data: BinanceBookTickerData,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BinanceBookTickerData {
    pub E: Option<i64>,
    pub s: String,
    pub b: String,
    pub B: String,
    pub a: String,
    pub A: String,
}

impl BinanceBookTicker {
    pub fn stream(&self) -> &String {
        &self.stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_bookticker() {
        let s = r#"{
                        "stream": "bnbusdt@bookTicker",
                        "data":{
                            "u":400900217,    
                            "s":"BNBUSDT",    
                            "b":"25.35190000", 
                            "B":"31.21000000", 
                            "a":"25.36520000", 
                            "A":"40.66000000"  
                        }
                    }"#;
        let bookticker: BinanceBookTicker = serde_json::from_str(s).unwrap();
        assert_eq!(bookticker.stream, "bnbusdt@bookTicker");
        assert_eq!(bookticker.data.s, "BNBUSDT");
        assert_eq!(bookticker.data.b, "25.35190000");
        assert_eq!(bookticker.data.B, "31.21000000");
        assert_eq!(bookticker.data.a, "25.36520000");
        assert_eq!(bookticker.data.A, "40.66000000");
    }
}

use cryptoflow::chat::{ErrorResponse, Response};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;
use tungstenite::Message;
pub struct Subscriber {
    symbols: HashSet<String>,
    tx: UnboundedSender<Message>,
    /// 发送到交易所的请求id与策略放请求的映射
    ids: HashMap<i64, i64>,
}

impl Subscriber {
    pub fn new(tx: UnboundedSender<Message>) -> Self {
        Self {
            symbols: HashSet::default(),
            tx,
            ids: HashMap::default(),
        }
    }

    pub fn on_exchange_response<T: Serialize>(
        &mut self,
        mut response: Response<T>,
    ) -> anyhow::Result<()> {
        if let Some(id) = self.ids.remove(&response.id) {
            response.id = id;
            self.tx
                .send(Message::Text(serde_json::to_string(&response)?.into()))?;
        }
        Ok(())
    }

    pub fn on_exchange_error(&mut self, mut response: ErrorResponse) -> anyhow::Result<()> {
        if let Some(id) = self.ids.remove(&response.id) {
            response.id = id;
            self.tx
                .send(Message::Text(serde_json::to_string(&response)?.into()))?;
        }
        Ok(())
    }

    pub fn on_strategy_client_subscribe(&mut self, id: i64, req: i64, symbols: Vec<String>) {
        self.ids.insert(id, req);
        self.symbols.extend(symbols);
    }

    pub fn is_subscribed(&self, symbol: &String) -> bool {
        self.symbols.contains(symbol)
    }

    pub fn forward_to_strategy_client(&self, data: &String) -> anyhow::Result<()> {
        tracing::info!("forward data: {:?}", data);
        self.tx.send(Message::Text(data.clone().into()))?;
        Ok(())
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, std::string::String> {
        self.symbols.iter()
    }
}

use serde_json::Value;
use tracing::{error, info};
use tungstenite::Message;
use url::Url;
use websocket::BinanceWsApiWebsocketClient;
use websocket::Credentials;

pub struct Account {
    rx: Option<tokio::sync::mpsc::Receiver<Value>>,
    disconnected: bool,
}

impl Account {
    pub async fn new(addr: &str, cred: Credentials) -> anyhow::Result<Self> {
        let addr = Url::parse(addr)?.to_string();

        info!("Account Websocket: {:?}", addr);
        let mut ws = BinanceWsApiWebsocketClient::new_private(cred);
        ws.set_url(&addr);
        let rx = ws.connect().await?;
        Ok(Self {
            rx: Some(rx),
            disconnected: false,
        })
    }

    pub fn disconnected(&self) -> bool {
        self.disconnected
    }

    pub async fn process(&mut self) -> anyhow::Result<Option<Message>> {
        if let Some(rx) = self.rx.as_mut() {
            match rx.recv().await {
                Some(inner) => match &inner {
                    Value::String(s) => return Ok(Some(Message::Text(s.clone().into()))),
                    _ => (),
                },
                None => {
                    if !self.disconnected {
                        error!("account disconnected");
                        self.disconnected = true
                    }
                }
            }
        }
        Ok(None)
    }

    /// FIXME:
    pub async fn reconnect(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

use futures_util::{SinkExt, TryStreamExt, stream::FusedStream};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{self, Message, client::IntoClientRequest},
};
use tracing::info;

pub struct WebSocketClient {
    inner: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebSocketClient {
    pub async fn new(addr: &str) -> anyhow::Result<Self> {
        let request = addr.into_client_request()?;
        match tokio::time::timeout(Duration::from_secs(3), async {
            tokio_tungstenite::connect_async(request).await
        })
        .await
        {
            Ok(Ok((ws, _))) => {
                info!("Connect to {}", addr);
                return Ok(Self { inner: ws });
            }
            Ok(Err(e)) => {
                return Err(anyhow::anyhow!("Connection error({})", e));
            }
            Err(_) => {
                return Err(anyhow::anyhow!("Connection timeout({})", addr));
            }
        };
    }

    pub async fn recv(&mut self) -> anyhow::Result<Option<Message>> {
        return Ok(self.inner.try_next().await?);
    }

    pub async fn send(&mut self, msg: Message) -> anyhow::Result<()> {
        self.inner.send(msg).await?;
        Ok(())
    }

    pub async fn close(
        &mut self,
        msg: Option<tungstenite::protocol::CloseFrame>,
    ) -> anyhow::Result<()> {
        self.inner.close(msg).await?;
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.inner.is_terminated()
    }
}

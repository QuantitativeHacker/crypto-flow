use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    net::SocketAddr,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use tracing::info;
use url::Url;

pub type TcpStreamSender = WebSocketSender<SplitSink<WebSocketStream<TcpStream>, Message>, Message>;
pub type TcpStreamReceiver = WebSocketReceiver<SplitStream<WebSocketStream<TcpStream>>>;
pub type Connection = (
    SocketAddr,
    UnboundedSender<Message>,
    UnboundedReceiver<Message>,
);

#[derive(Debug)]
pub struct WebSocketReceiver<T>
where
    T: StreamExt + Unpin + Debug,
    T::Item: Debug,
{
    peer: SocketAddr,
    inner: T,
}

impl<T> WebSocketReceiver<T>
where
    T: StreamExt + Unpin + Debug,
    T::Item: Debug,
{
    pub fn new(peer: SocketAddr, inner: T) -> Self {
        Self { peer, inner }
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.peer
    }

    pub async fn recv(&mut self) -> Option<T::Item> {
        let msg = self.inner.next().await;
        msg
    }
}

#[derive(Debug)]
pub struct WebSocketSender<T, Item>
where
    T: SinkExt<Item> + Unpin,
    Item: Debug + Display,
    <T as futures_util::Sink<Item>>::Error: std::error::Error + Send + Sync + 'static,
{
    peer: SocketAddr,
    inner: T,
    phantomdata: PhantomData<Item>,
}

impl<T, Item> WebSocketSender<T, Item>
where
    T: SinkExt<Item> + Unpin,
    Item: Debug + Display,
    <T as futures_util::Sink<Item>>::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn new(peer: SocketAddr, inner: T) -> Self {
        Self {
            peer,
            inner,
            phantomdata: PhantomData,
        }
    }

    pub async fn send(&mut self, msg: Item) -> anyhow::Result<()> {
        self.inner.send(msg).await?;
        Ok(())
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.peer
    }

    pub async fn close(&mut self) -> anyhow::Result<()> {
        self.inner.close().await?;
        Ok(())
    }
}

pub struct WebSocketServer {
    inner: TcpListener,
}

impl WebSocketServer {
    pub async fn new(addr: &str) -> anyhow::Result<Self> {
        let addr = Url::parse(addr)?;
        let listener = TcpListener::bind(format!(
            "{}:{}",
            addr.host_str().expect("Invalid host"),
            addr.port().expect("Invalid port")
        ))
        .await?;
        info!("Bind address {}", addr);

        Ok(Self { inner: listener })
    }

    pub async fn accept(&self) -> anyhow::Result<(SocketAddr, TcpStreamSender, TcpStreamReceiver)> {
        let (stream, peer) = self.inner.accept().await?;

        // let peer = peer.to_string();
        info!("Peer address connect: {}", peer);
        let ws = tokio_tungstenite::accept_async(stream).await?;
        let (write, read) = ws.split();

        return Ok((
            peer.clone(),
            WebSocketSender::new(peer.clone(), write),
            WebSocketReceiver::new(peer.clone(), read),
        ));
    }
}

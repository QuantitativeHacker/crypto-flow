mod trade;

use crate::rest::Rest;
use binance::{event_handlers::DefaultUserDataHandler, *};
use clap::Parser;
use cryptoflow::init_tracing;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};
use trade::UsdtTrade;
use websocket::Credentials;

#[derive(Debug, Deserialize)]
struct Config {
    apikey: String,
    pem: String,
    local: String,
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, help = "Config path")]
    config: String,
    #[arg(short, long, default_value_t = tracing::Level::INFO)]
    level: tracing::Level,
}

impl Args {
    pub fn load(&self) -> anyhow::Result<Config> {
        info!("Load config from {}", self.config);
        let buf = std::fs::read_to_string(self.config.clone())?;
        let config: Config = native_json::parse(&buf)?;
        Ok(config)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = args.load()?;

    let path = std::env::current_exe()?;
    let filename = match path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => "unknown".into(),
    };

    let _guard = init_tracing(&filename, "log", &args.level.to_string().to_lowercase())?;

    let app = Application::new(&config.local).await?;
    let market = Market::new().await?;

    let rest = Arc::new(Rest::new(
        "https://fapi.binance.com",
        &config.apikey,
        &config.pem,
        3000,
    )?);

    let credentials = Credentials::new(config.apikey, config.pem, "".to_string(), "0");
    let account = Account::new(&credentials, DefaultUserDataHandler).await;
    let trade = UsdtTrade::new(rest.clone(), account).await?;

    if let Err(e) = app.keep_running(market, trade).await {
        error!("{}", e);
    }

    Ok(())
}

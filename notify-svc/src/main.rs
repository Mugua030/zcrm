use anyhow::Result;
use notify_svc::{AppConfig, NotifyService};
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load().expect("Failed to load config");
    let port = config.server.port;
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    info!("notify listening on {}", addr);

    let svc = NotifyService::new(config).into_server();
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}

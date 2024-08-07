use anyhow::Result;
use metadata_svc::{AppConfig, MetadataService};
use tonic::transport::Server;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let conf = AppConfig::load().expect("fail to load config");
    let port = conf.server.port;
    let addr = format!("[::1]:{}", port).parse().unwrap();

    let svc = MetadataService::new(conf).into_server();
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

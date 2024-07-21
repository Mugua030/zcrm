use anyhow::Result;
use tonic::transport::Server;
use user_stat_svc::{AppConfig, UserStatsService};

#[tokio::main]
async fn main() -> Result<()> {
    let conf = AppConfig::load().expect("fail to load app config file");
    println!("port: {}", conf.server.port);

    // tonic server (grpc sever)

    let addr = format!("[::1]:{}", conf.server.port).parse().unwrap();
    let svc = UserStatsService::new(conf).await.into_server();
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

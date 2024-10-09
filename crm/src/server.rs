use std::mem;

use anyhow::Result;
use crm::pb::{user_service_server::UserService, CreateUserRequest, GetUserRequest, User};

use crm::{AppConfig, CrmService};
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::{async_trait, Request, Response, Status};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Default)]
pub struct UserServer {}

#[async_trait]
impl UserService for UserServer {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("get user: {:?}", input);
        Ok(Response::new(User::default()))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("create user: {:?}", input);
        let user = User::new(1, &input.name, &input.email);
        Ok(Response::new(user))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    // user server test
    /*
    let addr = "[::1]:50051".parse().unwrap();
    let svc = UserServer::default();

    println!("[UserService] listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
    */

    // crm service
    let mut config = AppConfig::load().expect("Failed to load config");
    let port = config.server.port;
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    info!("crm service listening on {}", addr);

    let tls = mem::take(&mut config.server.tls);
    let svc = CrmService::try_new(config).await?.into_server()?;
    info!("Begin TLS sever...");

    // TLS
    if let Some(tls) = tls {
        println!("tls run...");
        let identity = Identity::from_pem(tls.cert, tls.key);
        Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(svc)
            .serve(addr)
            .await?;
    } else {
        // No TLS
        println!("no TLS run...");
        Server::builder().add_service(svc).serve(addr).await?;
    }

    Ok(())
}

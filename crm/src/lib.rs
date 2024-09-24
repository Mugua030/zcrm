mod abi;
mod config;

pub mod pb;

use anyhow::Result;
use metadata_svc::metadata_client::MetadataClient;
use notify_svc::pb::notify_client::NotifyClient;
use pb::{
    crm_server::{Crm, CrmServer},
    WelcomeRequest, WelcomeResponse,
};

pub use config::AppConfig;
use tonic::{async_trait, transport::Channel, Request, Response, Status};
use user_stat_svc::pb::user_stats_client::UserStatsClient;

pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    notify: NotifyClient<Channel>,
    metadata: MetadataClient<Channel>,
}

#[async_trait]
impl Crm for CrmService {
    async fn welcome(
        &self,
        request: Request<WelcomeRequest>,
    ) -> Result<Response<WelcomeResponse>, Status> {
        self.welcome(request.into_inner()).await
    }
}

impl CrmService {
    pub async fn try_new(config: AppConfig) -> Result<Self> {
        dbg!("config: ", &config);
        let user_stats = UserStatsClient::connect(config.server.user_stats_svc.clone()).await?;
        let notify = NotifyClient::connect(config.server.notify_svc.clone()).await?;
        let metadata = MetadataClient::connect(config.server.metadata_svc.clone()).await?;

        Ok(Self {
            config,
            user_stats,
            notify,
            metadata,
        })
    }

    pub fn into_server(self) -> CrmServer<Self> {
        CrmServer::new(self)
    }
}

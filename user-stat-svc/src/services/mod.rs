mod user_stats_svc;
pub mod user_stats_svc_test;

use crate::config::*;

use sqlx::PgPool;

use crate::pb::User;
use futures::Stream;
use std::{pin::Pin, sync::Arc};
use tonic::{Response, Status};

pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;
pub type ServiceResult<T> = Result<Response<T>, Status>;

pub struct UserStatsService {
    pub inner: Arc<UserStatsServiceInner>,
}

pub struct UserStatsServiceInner {
    pub config: AppConfig,
    pub pool: PgPool,
}

impl UserStatsService {
    pub async fn new(conf: AppConfig) -> Self {
        let db_pool = PgPool::connect(&conf.server.db_url)
            .await
            .expect("failed to connect db");

        let inner = UserStatsServiceInner {
            config: conf,
            pool: db_pool,
        };
        Self {
            inner: Arc::new(inner),
        }
    }
}

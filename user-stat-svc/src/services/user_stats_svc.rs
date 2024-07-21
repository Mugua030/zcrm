use super::{UserStatsService, UserStatsServiceInner};
use std::ops::Deref;

use super::{ResponseStream, ServiceResult};
use tonic::{async_trait, Request};

use crate::pb::{
    user_stats_server::{UserStats, UserStatsServer},
    QueryRequest, RawQueryRequest,
};

impl UserStatsService {
    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    type RawQueryStream = ResponseStream;

    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();
        self.query(query).await
    }

    async fn raw_query(
        &self,
        req: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = req.into_inner();
        self.raw_query(query).await
    }
}

use crate::config::AppConfig;
use std::pin::Pin;

use futures::Stream;
use tonic::{async_trait, Request, Response, Status, Streaming};

pub use crate::pb::{
    metadata_server::{Metadata, MetadataServer},
    Content, MaterializeRequest,
};

#[allow(unused)]
pub struct MetadataService {
    config: AppConfig,
}

pub type ServiceResult<T> = Result<Response<T>, Status>;
pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content, Status>> + Send>>;

#[async_trait]
impl Metadata for MetadataService {
    type MaterializeStream = ResponseStream;

    async fn materialize(
        &self,
        request: Request<Streaming<MaterializeRequest>>,
    ) -> ServiceResult<Self::MaterializeStream> {
        let query_stream = request.into_inner();

        self.materialize(query_stream).await
    }
}

impl MetadataService {
    pub fn new(config: AppConfig) -> Self {
        MetadataService { config }
    }

    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}

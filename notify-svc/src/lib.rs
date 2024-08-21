mod abi;
mod config;
pub mod pb;

use futures::Stream;
use std::{pin::Pin, sync::Arc};

pub use config::AppConfig;
use pb::{notify_server::Notify, send_request::Msg, SendRequest, SendResponse};

use tokio::sync::mpsc;
use tonic::{async_trait, Request, Response, Status, Streaming};

#[derive(Clone)]
pub struct NotifyService {
    inner: Arc<NotifyServiceInner>,
}

#[allow(unused)]
pub struct NotifyServiceInner {
    config: AppConfig,
    sender: mpsc::Sender<Msg>,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<SendResponse, Status>> + Send>>;

#[async_trait]
impl Notify for NotifyService {
    type SendMsgStream = ResponseStream;

    async fn send_msg(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> Result<Response<Self::SendMsgStream>, Status> {
        let stm = request.into_inner();
        self.send(stm).await
    }
}

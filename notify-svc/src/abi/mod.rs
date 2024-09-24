mod email;
mod in_app;
mod sms;

use chrono::Utc;
use futures::{Stream, StreamExt};
use metadata_svc::{pb::Content, Tpl};
use prost_types::Timestamp;
use std::{ops::Deref, sync::Arc, time::Duration};
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    pb::{notify_server::NotifyServer, send_request::Msg, EmailMsg, SendRequest, SendResponse},
    AppConfig, NotifyService, NotifyServiceInner, ResponseStream, ServiceResult,
};

const CHANNEL_SIZE: usize = 1024;
pub trait Sender {
    async fn send(self, svc: NotifyService) -> Result<SendResponse, Status>;
}

impl NotifyService {
    pub fn new(config: AppConfig) -> Self {
        let sender = dummy_send();
        let inner = NotifyServiceInner { config, sender };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> NotifyServer<Self> {
        NotifyServer::new(self)
    }

    pub async fn send(
        &self,
        mut stm: impl Stream<Item = Result<SendRequest, Status>> + Send + 'static + Unpin,
    ) -> ServiceResult<ResponseStream> {
        info!("[NotifyService] send running - begin");
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        let ntf = self.clone();

        tokio::spawn(async move {
            while let Some(Ok(req)) = stm.next().await {
                info!("[NotifyService.spawn] req {:?}", &req);
                let ntf_clone = ntf.clone();
                let res = match req.msg {
                    Some(Msg::Email(email)) => {
                        info!("sended-msg: {:?}", &email);
                        email.send(ntf_clone).await
                    }
                    Some(Msg::Sms(sms)) => sms.send(ntf_clone).await,
                    Some(Msg::InAppMsg(in_app)) => in_app.send(ntf_clone).await,
                    None => {
                        warn!("Invalid request");
                        Err(Status::invalid_argument("invalid request"))
                    }
                };
                tx.send(res).await.unwrap();
            }
        });

        let stm = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stm)))
    }
}

impl Deref for NotifyService {
    type Target = NotifyServiceInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl SendRequest {
    pub fn new(
        subject: String,
        sender: String,
        recipients: &[String],
        contents: &[Content],
    ) -> Self {
        let tpl = Tpl(contents);
        let msg = Msg::Email(EmailMsg {
            msg_id: Uuid::new_v4().to_string(),
            subject,
            sender,
            recipients: recipients.to_vec(),
            body: tpl.to_body(),
        });

        SendRequest { msg: Some(msg) }
    }
}

fn dummy_send() -> mpsc::Sender<Msg> {
    let (tx, mut rx) = mpsc::channel(CHANNEL_SIZE * 100);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            info!("Sending msg: {:?}", msg);
            sleep(Duration::from_millis(300)).await;
        }
    });
    tx
}

fn to_ts() -> Timestamp {
    let now = Utc::now();
    Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pb::{EmailMsg, InAppMsg, SmsMsg},
        AppConfig,
    };
    use anyhow::Result;

    #[tokio::test]
    async fn test_send() -> Result<()> {
        let config = AppConfig::load()?;
        let svc = NotifyService::new(config);
        let stream = tokio_stream::iter(vec![
            Ok(EmailMsg::fake().into()),
            Ok(SmsMsg::fake().into()),
            Ok(InAppMsg::fake().into()),
        ]);

        let response = svc.send(stream).await?;
        let ret = response.into_inner().collect::<Vec<_>>().await;
        assert_eq!(ret.len(), 3);

        Ok(())
    }
}

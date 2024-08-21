use super::{to_ts, Sender};
use crate::{
    pb::{send_request::Msg, InAppMsg, SendRequest, SendResponse},
    NotifyService,
};
use tonic::Status;
use tracing::warn;

impl Sender for InAppMsg {
    async fn send(self, svc: NotifyService) -> Result<SendResponse, Status> {
        let msg_id = self.msg_id.clone();
        svc.sender.send(Msg::InAppMsg(self)).await.map_err(|e| {
            warn!("Failed to send msg: {:?}", e);
            Status::internal("Failed to send msg")
        })?;

        Ok(SendResponse {
            msg_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<InAppMsg> for Msg {
    fn from(in_app: InAppMsg) -> Self {
        Msg::InAppMsg(in_app)
    }
}

impl From<InAppMsg> for SendRequest {
    fn from(in_app: InAppMsg) -> Self {
        let msg: Msg = in_app.into();
        SendRequest { msg: Some(msg) }
    }
}

#[cfg(feature = "test_utils")]
impl InAppMsg {
    pub fn fake() -> Self {
        use uuid::Uuid;
        InAppMsg {
            msg_id: Uuid::new_v4().to_string(),
            device_id: Uuid::new_v4().to_string(),
            title: "Hi".to_string(),
            body: "Hi, Girl".to_string(),
        }
    }
}

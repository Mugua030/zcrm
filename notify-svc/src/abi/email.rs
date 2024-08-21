use super::{to_ts, Sender};
use crate::{
    pb::{send_request::Msg, EmailMsg, SendRequest, SendResponse},
    NotifyService,
};
use tonic::Status;
use tracing::warn;

impl Sender for EmailMsg {
    async fn send(self, svc: NotifyService) -> Result<SendResponse, Status> {
        let msg_id = self.msg_id.clone();
        svc.sender.send(Msg::Email(self)).await.map_err(|e| {
            warn!("Failed to send msg: {:?}", e);
            Status::internal("Failed to send msg")
        })?;

        Ok(SendResponse {
            msg_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<EmailMsg> for Msg {
    fn from(value: EmailMsg) -> Self {
        Msg::Email(value)
    }
}

impl From<EmailMsg> for SendRequest {
    fn from(value: EmailMsg) -> Self {
        let msg: Msg = value.into();
        SendRequest { msg: Some(msg) }
    }
}

#[cfg(feature = "test_utils")]
impl EmailMsg {
    pub fn fake() -> Self {
        use fake::faker::internet::en::SafeEmail;
        use fake::Fake;
        use uuid::Uuid;
        EmailMsg {
            msg_id: Uuid::new_v4().to_string(),
            sender: SafeEmail().fake(),
            recipients: vec![SafeEmail().fake()],
            subject: "Hi".to_string(),
            body: "Hi, Girl".to_string(),
        }
    }
}

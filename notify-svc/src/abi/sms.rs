use super::{to_ts, Sender};
use crate::{
    pb::{send_request::Msg, SendRequest, SendResponse, SmsMsg},
    NotifyService,
};

use tonic::Status;
use tracing::warn;

impl Sender for SmsMsg {
    async fn send(self, svc: NotifyService) -> Result<SendResponse, Status> {
        let msg_id = self.msg_id.clone();
        svc.sender.send(Msg::Sms(self)).await.map_err(|e| {
            warn!("Failed to send msg: {:?}", e);
            Status::internal("Failed to send msg")
        })?;

        Ok(SendResponse {
            msg_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<SmsMsg> for Msg {
    fn from(value: SmsMsg) -> Self {
        Msg::Sms(value)
    }
}

impl From<SmsMsg> for SendRequest {
    fn from(sms: SmsMsg) -> Self {
        let msg: Msg = sms.into();
        SendRequest { msg: Some(msg) }
    }
}

#[cfg(feature = "test_utils")]
impl SmsMsg {
    pub fn fake() -> Self {
        use fake::faker::phone_number::en::PhoneNumber;
        use fake::Fake;
        use uuid::Uuid;
        SmsMsg {
            msg_id: Uuid::new_v4().to_string(),
            sender: PhoneNumber().fake(),
            recipients: vec![PhoneNumber().fake()],
            body: "Hello world!".to_string(),
        }
    }
}

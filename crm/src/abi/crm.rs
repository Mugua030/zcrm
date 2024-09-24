use crate::{
    pb::{WelcomeRequest, WelcomeResponse},
    CrmService,
};
use notify_svc::pb::SendRequest;

use futures::StreamExt;
use metadata_svc::pb::{Content, MaterializeRequest};

use crate::pb::User;
use chrono::{Duration, Utc};
use prost_types::Timestamp;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::warn;
use user_stat_svc::pb::QueryRequest;

impl User {
    pub fn new(id: u64, name: &str, email: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        User {
            id,
            name: name.to_string(),
            email: email.to_string(),
            created_at: Some(Timestamp {
                seconds: now.as_secs() as i64,
                nanos: now.subsec_nanos() as i32,
            }),
        }
    }
}

impl CrmService {
    pub async fn welcome(&self, req: WelcomeRequest) -> Result<Response<WelcomeResponse>, Status> {
        let request_id = req.id;
        let d1 = Utc::now() - Duration::days(req.interval as _);
        let d2 = d1 + Duration::days(1);
        let query = QueryRequest::new_with_dt("created_at", d1, d2);
        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        // filter users that adapted
        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&req.content_ids))
            .await?
            .into_inner();

        let contents: Vec<Content> = contents
            .filter_map(|v| async move { v.ok() })
            .collect()
            .await;
        let contents = Arc::new(contents);

        // send msg
        let (tx, rx) = mpsc::channel(1024);
        let sender = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = res_user_stats.next().await {
                let contents = contents.clone();
                let sender = sender.clone();
                let tx = tx.clone();

                let req = SendRequest::new("Welcome".to_string(), sender, &[user.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message:  {:?}", e);
                }
            }
        });

        // receive
        let reqs = ReceiverStream::new(rx);
        self.notify.clone().send_msg(reqs).await?;

        Ok(Response::new(WelcomeResponse { id: request_id }))
    }
}

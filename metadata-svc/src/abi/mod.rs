use futures::{stream, Stream, StreamExt};
use std::collections::HashSet;

use crate::{
    metadata_svc::{MetadataService, ResponseStream, ServiceResult},
    pb::{Content, MaterializeRequest, Publisher},
};

use chrono::{DateTime, Days, Utc};
use fake::{
    faker::{chrono::en::DateTimeBetween, lorem::en::Sentence, name::en::Name},
    Fake, Faker,
};
use prost_types::Timestamp;
use rand::Rng;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Response;

const CHANNEL_SIZE: usize = 1024;
impl MetadataService {
    pub async fn materialize(
        &self,
        mut stream: impl Stream<Item = Result<MaterializeRequest, tonic::Status>>
            + Send
            + 'static
            + Unpin,
    ) -> ServiceResult<ResponseStream> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        tokio::spawn(async move {
            while let Some(Ok(req)) = stream.next().await {
                let content = Content::materialize(req.id);
                tx.send(Ok(content)).await.unwrap();
            }
        });

        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

impl Content {
    pub fn materialize(id: u32) -> Self {
        let mut rng = rand::thread_rng();
        Content {
            id,
            name: Name().fake(),
            description: Sentence(3..7).fake(),
            publishers: (1..rng.gen_range(2..10))
                .map(|_| Publisher::new())
                .collect(),
            url: "https://placehold.co/600x300".to_string(),
            image: "https://placehold.co/600x300".to_string(),
            typ: Faker.fake(),
            created_at: created_at(),
            views: rng.gen_range(123432..10000000),
            likes: rng.gen_range(1234..100000),
            dislikes: rng.gen_range(123..10000),
        }
    }

    pub fn to_body(&self) -> String {
        format!("Content: {:?}", self)
    }
}

impl Publisher {
    pub fn new() -> Self {
        Publisher {
            id: (10000..2000000).fake(),
            name: Name().fake(),
            avatar: "https://placehold.co/500x500".to_string(),
        }
    }
}

// tpl
pub struct Tpl<'a>(pub &'a [Content]);
impl<'a> Tpl<'a> {
    pub fn to_body(&self) -> String {
        format!("Tpl: {:?}", self.0)
    }
}

impl MaterializeRequest {
    pub fn new_with_ids(ids: &[u32]) -> impl Stream<Item = Self> {
        let reqs: HashSet<_> = ids.iter().map(|id| Self { id: *id }).collect();
        stream::iter(reqs)
    }
}

fn before(days: u64) -> DateTime<Utc> {
    Utc::now().checked_sub_days(Days::new(days)).unwrap()
}

fn created_at() -> Option<Timestamp> {
    let date: DateTime<Utc> = DateTimeBetween(before(365), before(0)).fake();
    Some(Timestamp {
        seconds: date.timestamp(),
        nanos: date.timestamp_subsec_nanos() as i32,
    })
}

// for test
// 自定义流类型，将 ReceiverStream 包装为符合 tonic::Streaming 预期的类型

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppConfig;
    use anyhow::Result;

    #[tokio::test]
    async fn test_materialize() -> Result<()> {
        let conf = AppConfig::load()?;
        let svc = MetadataService::new(conf);

        // 创建流
        let stm = tokio_stream::iter(vec![
            Ok(MaterializeRequest { id: 1 }),
            Ok(MaterializeRequest { id: 2 }),
            Ok(MaterializeRequest { id: 3 }),
        ]);

        let resp = svc.materialize(stm).await?;
        let ret = resp.into_inner().collect::<Vec<_>>().await;
        assert_eq!(ret.len(), 3);

        for re in ret.into_iter().flatten() {
            println!("con={:?}", re);
        }

        Ok(())
    }
}

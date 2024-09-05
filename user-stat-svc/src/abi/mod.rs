use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;
use prost_types::Timestamp;
use tonic::{Response, Status};

use crate::pb::{QueryRequest, RawQueryRequest, User};

use crate::services::{ResponseStream, ServiceResult, UserStatsService};

impl UserStatsService {
    pub async fn query(&self, query: QueryRequest) -> ServiceResult<ResponseStream> {
        let mut sql = "SELECT email, name FROM user_stats WHERE ".to_string();

        let time_conditions = query
            .timestamps
            .into_iter()
            .map(|(k, v)| timestamp_query(&k, v.lower, v.upper))
            .join(" AND ");

        sql.push_str(&time_conditions);

        let id_conditions = query
            .ids
            .into_iter()
            .map(|(k, v)| ids_query(&k, v.ids))
            .join(" AND ");

        //sql.push_str(" AND ");
        sql.push_str(&id_conditions);

        // limit
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        self.raw_query(RawQueryRequest { query: sql }).await
    }

    pub async fn raw_query(&self, rawreq: RawQueryRequest) -> ServiceResult<ResponseStream> {
        println!("Q-SQL: {:?}", &rawreq.query);
        let Ok(ret) = sqlx::query_as::<_, User>(&rawreq.query)
            .fetch_all(&self.inner.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "failed to fetch data: {}",
                rawreq.query
            )));
        };
        // success
        Ok(Response::new(Box::pin(futures::stream::iter(
            ret.into_iter().map(Ok),
        ))))
    }
}

fn timestamp_query(name: &str, lower: Option<Timestamp>, upper: Option<Timestamp>) -> String {
    if lower.is_none() && upper.is_none() {
        return "TRUE".to_string();
    }

    if lower.is_none() {
        let upper = ts_to_utc(upper.unwrap());
        return format!("{} <= '{}'", name, upper.to_rfc3339());
    }

    if upper.is_none() {
        let lower = ts_to_utc(lower.unwrap());
        return format!("{} >= '{}'", name, lower.to_rfc3339());
    }

    format!(
        "{} BETWEEN '{}' AND '{}'",
        name,
        ts_to_utc(upper.unwrap()).to_rfc3339(),
        ts_to_utc(lower.unwrap()).to_rfc3339()
    )
}

fn ts_to_utc(ts: Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as _).unwrap()
}
fn ids_query(name: &str, ids: Vec<u32>) -> String {
    if ids.is_empty() {
        return "TRUE".to_string();
    }

    format!("array{:?} <@ {}", ids, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use futures::StreamExt;

    use crate::AppConfig;

    #[tokio::test]
    async fn test_raw_query() -> Result<()> {
        let conf = AppConfig::load().expect("failed to load config");
        let svc = UserStatsService::new(conf).await;
        let mut stream = svc
            .raw_query(RawQueryRequest {
                query: "select * from user_stats where created_at > '2021-01-01' limit 6"
                    .to_string(),
            })
            .await?
            .into_inner();

        // show
        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }

        Ok(())
    }
}

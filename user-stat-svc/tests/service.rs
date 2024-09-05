use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use futures::StreamExt;
use sqlx_db_tester::TestPg;
use tokio::time::sleep;
use tonic::transport::Server;
use user_stat_svc::{
    pb::{user_stats_client::UserStatsClient, QueryRequestBuilder, RawQueryRequestBuilder},
    services::user_stats_svc_test::test_utils,
    UserStatsService,
};

const PORT_BASE: u32 = 60000;

#[tokio::test]
async fn test_raw_query() -> Result<()> {
    let (_tdb, addr) = start_server(PORT_BASE).await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}")).await?;
    let req = RawQueryRequestBuilder::default()
        .query("SELECT * FROM user_stats WHERE created_at > '2024-01-01' LIMIT 5")
        .build()?;

    let stm = client.raw_query(req).await?.into_inner();
    let ret = stm
        .then(|res| async move { res.unwrap() })
        .collect::<Vec<_>>()
        .await;

    assert_eq!(ret.len(), 5);

    Ok(())
}

#[tokio::test]
async fn test_query() -> Result<()> {
    let (_tdb, addr) = start_server(PORT_BASE + 1).await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}")).await?;
    let query = QueryRequestBuilder::default()
        .timestamp(("created_at".to_string(), test_utils::tq(Some(1320), None)))
        .timestamp((
            "last_visited_at".to_string(),
            test_utils::tq(Some(1310), None),
        ))
        //.id((
        //    "viewed_but_not_started".to_string(),
        //    test_utils::id(&[201486]),
        //))
        .limit(3)
        .build()
        .unwrap();

    println!("query: {:?}", query);
    let stm = client.query(query).await?.into_inner();
    let ret = stm.collect::<Vec<_>>().await;

    for user in ret.iter().flatten() {
        println!("user: {:?}\n", user);
    }

    assert_eq!(ret.len(), 3);

    Ok(())
}

async fn start_server(port: u32) -> Result<(TestPg, SocketAddr)> {
    let addr = format!("[::1]:{}", port).parse()?;
    let (tdb, svc) = UserStatsService::new_for_test().await?;

    tokio::spawn(async move {
        Server::builder()
            .add_service(svc.into_server())
            .serve(addr)
            .await
            .unwrap();
    });
    sleep(Duration::from_micros(1)).await;

    Ok((tdb, addr))
}

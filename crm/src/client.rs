use anyhow::Result;
//use crm::pb::{user_service_client::UserServiceClient, CreateUserRequest};
use crm::pb::{crm_client::CrmClient, WelcomeRequestBuilder};
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig},
    Request,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    /*
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(CreateUserRequest {
        name: "C.Martin".to_string(),
        email: "martin@abcxxx.test.org".to_string(),
    });

    // invoke the method defined by protobuf
    let resp = client.create_user(request).await?;
    println!("resp: {:?}", resp);
    */

    // TLS
    let pem = include_str!("../../fixtures/rootCA.pem");
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(pem))
        .domain_name("localhost");

    let channel = Channel::from_static("https://0.0.0.0:50000")
        .tls_config(tls)?
        .connect()
        .await
        .expect("channel fail");

    let token = include_str!("../../fixtures/token").trim();
    //dbg!(token);
    let token: MetadataValue<_> = format!("Bearer {}", token).parse()?; //.expect("token parse to metavalue fail");
                                                                        //println!("meta-token {:?}", token);

    //let mut client = CrmClient::connect("http://0.0.0.0:8081").await?;
    let mut client = CrmClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });

    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(90u32)
        .content_ids([1u32, 2, 3])
        .build()?;

    println!("Begin invoke rpc...");
    let resp = client.welcome(Request::new(req)).await?.into_inner();
    println!("Resp: {:?}", resp);

    Ok(())
}

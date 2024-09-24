use anyhow::Result;
//use crm::pb::{user_service_client::UserServiceClient, CreateUserRequest};
use crm::pb::{crm_client::CrmClient, WelcomeRequestBuilder};
use tonic::Request;
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

    let mut client = CrmClient::connect("http://0.0.0.0:50000").await?;

    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(90u32)
        .content_ids([1u32, 2, 3])
        .build()?;

    let resp = client.welcome(Request::new(req)).await?.into_inner();
    println!("Resp: {:?}", resp);

    Ok(())
}

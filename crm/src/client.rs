use anyhow::Result;
use crm::pb::{user_service_client::UserServiceClient, CreateUserRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(CreateUserRequest {
        name: "C.Martin".to_string(),
        email: "martin@abcxxx.test.org".to_string(),
    });

    // invoke the method defined by protobuf
    let resp = client.create_user(request).await?;
    println!("resp: {:?}", resp);

    Ok(())
}

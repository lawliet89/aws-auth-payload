//! This example uses the default AWS credentials priority and prints out the example
//! payload using HTTP POST and a pre-signed URL.
//!
//! Vault AWS IAM authentication uses the Post method while Kubernetes uses the pre-signed URL

use aws_auth_payload::client;
use log::info;
use rusoto_core::region::Region;

#[tokio::main]
async fn main() -> Result<(), aws_auth_payload::Error> {
    env_logger::init();
    info!("Retrieving AWS Credentials from the environment");

    let credentials = aws_auth_payload::get_aws_credentials().await?;
    let payload = client::AwsAuthIamPayload::new::<rusoto_core::Region>(
        &credentials,
        Some(Region::default()),
        Default::default(),
    );
    println!(
        "Post Payload: {}",
        serde_json::to_string_pretty(&payload)
            .map_err(|e| aws_auth_payload::Error::GenericError(e.to_string()))?
    );

    let header = [].iter().cloned().collect();
    println!(
        "Pre-signed URL: {}",
        client::presigned_url::<rusoto_core::Region>(&credentials, None, header, None)
    );
    Ok(())
}

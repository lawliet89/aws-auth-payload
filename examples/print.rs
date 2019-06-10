//! This example uses the default AWS credentials priority and prints out the example
//! payload using HTTP POST and a pre-signed URL.
//!
//! Vault AWS IAM authentication uses the Post method while Kubernetes uses the pre-signed URL

use aws_auth_payload::client;

fn main() -> Result<(), aws_auth_payload::Error> {
    println!("Retrieving AWS Credentials from the environment");
    let credentials = aws_auth_payload::get_aws_credentials()?;
    let payload = client::AwsAuthIamPayload::new::<rusoto_core::Region>(
        &credentials,
        None,
        Default::default(),
    );
    println!(
        "Post Payload: {}",
        serde_json::to_string_pretty(&payload)
            .map_err(|e| aws_auth_payload::Error::GenericError(e.to_string()))?
    );

    println!(
        "Pre-signed URL: {}",
        client::presigned_url::<rusoto_core::Region>(&credentials, None, Default::default(), None)
    );
    Ok(())
}

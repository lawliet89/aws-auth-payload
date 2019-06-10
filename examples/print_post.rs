//! This example uses the default AWS credentials priority and prints out the example
//! payload using HTTP POST.
//!
//! Vault AWS IAM authentication uses the Post method.

fn main() -> Result<(), aws_auth_payload::Error> {
    println!("Retrieving AWS Credentials from the environment");
    let credentials = aws_auth_payload::get_aws_credentials()?;
    let payload = aws_auth_payload::client::AwsAuthIamPayload::post::<rusoto_core::Region>(
        &credentials,
        None,
        Default::default(),
    );
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

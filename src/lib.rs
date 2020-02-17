pub mod client;
mod error;

#[doc(inline)]
pub use crate::error::Error;

use rusoto_core::credential::{AwsCredentials, DefaultCredentialsProvider, ProvideAwsCredentials};

/// Use the priority documented
/// [here](https://rusoto.github.io/rusoto/rusoto_credential/struct.ChainProvider.html)
/// obtain AWS credentials
///
/// # Limitations
///
/// Assuming roles through profiles is not supported at the moment. See this
/// [Github issue](https://github.com/rusoto/rusoto/issues/1120)
pub async fn get_aws_credentials() -> Result<AwsCredentials, Error> {
    let provider = DefaultCredentialsProvider::new()?;
    Ok(provider.credentials().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;

    #[tokio::test(threaded_scheduler)]
    async fn expected_aws_credentials() -> Result<(), crate::Error> {
        let access_key = "test_key";
        let secret_key = "test_secret";

        env::set_var("AWS_ACCESS_KEY_ID", access_key);
        env::set_var("AWS_SECRET_ACCESS_KEY", secret_key);

        let credentials = get_aws_credentials().await?;

        assert_eq!(credentials.aws_access_key_id(), access_key);
        assert_eq!(credentials.aws_secret_access_key(), secret_key);

        Ok(())
    }
}

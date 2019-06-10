pub mod client;
mod error;

#[doc(inline)]
pub use crate::error::Error;

use futures::future::Future as _;
use rusoto_core::credential::AwsCredentials;
use rusoto_core::{DefaultCredentialsProvider, ProvideAwsCredentials};

/// Use the priority documented
/// [here](https://rusoto.github.io/rusoto/rusoto_credential/struct.ChainProvider.html)
/// obtain AWS credentials
///
/// # Limitations
///
/// Assuming roles through profiles is not supported at the moment. See this
/// [Github issue](https://github.com/rusoto/rusoto/issues/1120)
pub fn get_aws_credentials() -> Result<AwsCredentials, Error> {
    let provider = DefaultCredentialsProvider::new()?;
    Ok(provider.credentials().wait()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;

    #[test]
    fn expected_aws_credentials() -> Result<(), crate::Error> {
        let access_key = "test_key";
        let secret_key = "test_secret";

        env::set_var("AWS_ACCESS_KEY_ID", access_key);
        env::set_var("AWS_SECRET_ACCESS_KEY", secret_key);

        let credentials = get_aws_credentials()?;

        assert_eq!(credentials.aws_access_key_id(), access_key);
        assert_eq!(credentials.aws_secret_access_key(), secret_key);

        Ok(())
    }
}

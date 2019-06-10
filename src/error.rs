use failure_derive::Fail;

/// Error type for this library
#[derive(Debug, Fail)]
pub enum Error {
    /// Errors related to retrieving AWS credentials
    #[fail(display = "Error retrieving AWS credentials: {}", _0)]
    CredentialsError(#[cause] rusoto_core::CredentialsError),
    /// Generic Error
    #[fail(display = "{}", _0)]
    GenericError(String),
}

impl From<rusoto_core::CredentialsError> for Error {
    fn from(error: rusoto_core::CredentialsError) -> Self {
        Error::CredentialsError(error)
    }
}

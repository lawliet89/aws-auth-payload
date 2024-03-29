use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::time::Duration;

use lazy_static::lazy_static;
use log::{debug, info};
use rusoto_core::credential::AwsCredentials;
use rusoto_core::param::{Params, ServiceParams};
use rusoto_core::signature::{SignedRequest, SignedRequestPayload};
use rusoto_core::Region;
use serde::{Deserialize, Serialize};

/// Payload for use to generate a payload for AWS IAM authentication
///
/// This payload is used by HashiCorp's Vault and is generated by making a POST request
/// to AWS STS `GetCallerIdentity`
///
/// See [Vault's Documentation](https://www.vaultproject.io/docs/auth/aws.html#iam-auth-method)
/// for more information.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct AwsAuthIamPayload {
    /// HTTP method used in the signed request. Currently only `POST` is supported
    pub iam_http_request_method: String,
    /// Base64-encoded HTTP URL used in the signed request
    pub iam_request_url: String,
    /// Base64-encoded body of the signed request
    pub iam_request_body: String,
    /// Headers of the signed request
    pub iam_request_headers: HashMap<String, Vec<String>>,
}

impl AwsAuthIamPayload {
    /// Creates a payload for use to generate a payload for AWS IAM authentication
    ///
    /// This payload is used by HashiCorp's Vault and is generated by making a POST request
    /// to AWS STS `GetCallerIdentity`
    ///
    /// See [Vault's Documentation](https://www.vaultproject.io/docs/auth/aws.html#iam-auth-method)
    /// for more information.
    /// If you do not provide a `region`, we will use a the "global" AWS STS endpoint.
    pub fn new<R>(
        credentials: &AwsCredentials,
        region: Option<R>,
        additional_headers: HashMap<&str, &str>,
    ) -> Self
    where
        R: Borrow<Region>,
    {
        info!("Building Login Payload for AWS authentication");
        let region = region
            .as_ref()
            .map(|r| Cow::Borrowed(r.borrow()))
            .unwrap_or_default();
        // Code below is referenced from the code for
        // https://rusoto.github.io/rusoto/rusoto_sts/trait.Sts.html#tymethod.get_caller_identity

        // Additional processing for Vault is referenced from Vault CLI's source code:
        // https://github.com/hashicorp/vault/blob/master/builtin/credential/aws/cli.go

        let mut request = SignedRequest::new("POST", "sts", &region, "/");
        let mut params = Params::new();

        params.put("Action", "GetCallerIdentity");
        params.put("Version", "2011-06-15");
        request.set_payload(Some(
            serde_urlencoded::to_string(&params).unwrap().into_bytes(),
        ));
        request.set_content_type("application/x-www-form-urlencoded".to_owned());

        for (header, value) in additional_headers.into_iter() {
            request.add_header(header, value)
        }

        request.sign(credentials);

        let uri = format!(
            "{}://{}{}",
            request.scheme(),
            request.hostname(),
            request.canonical_path()
        );

        let payload = match request.payload {
            Some(SignedRequestPayload::Buffer(ref buffer)) => base64::encode(buffer),
            _ => unreachable!("Payload was set above"),
        };

        // We need to convert the headers from bytes back into Strings...
        let headers = request
            .headers
            .iter()
            .map(|(k, v)| {
                let values = v
                    .iter()
                    .map(|v| unsafe { String::from_utf8_unchecked(v.to_vec()) })
                    .collect();

                (k.to_string(), values)
            })
            .collect();

        let result = Self {
            iam_http_request_method: "POST".to_string(),
            iam_request_url: base64::encode(&uri),
            iam_request_body: payload,
            iam_request_headers: headers,
        };

        debug!("AWS Payload: {:#?}", result);

        result
    }
}

/// Generates a pre-signed URL using the provided AWS Credentials to
/// AWS STS `GetCallerIdentity`
///
/// This is used by
/// [Kubernetes AWS IAM Authenticator](https://github.com/kubernetes-sigs/aws-iam-authenticator)
///
/// See [Vault's Documentation](https://www.vaultproject.io/docs/auth/aws.html#iam-auth-method)
/// for more information.
#[allow(clippy::implicit_hasher)]
pub fn presigned_url<R>(
    credentials: &AwsCredentials,
    region: Option<R>,
    additional_headers: HashMap<&str, &str>,
    expires_in: Option<&Duration>,
) -> String
where
    R: Borrow<Region>,
{
    lazy_static! {
        static ref DEFAULT_EXPIRES: Duration = Duration::from_secs(60);
    }

    info!("Building pre-signed URL for AWS authentication");
    let region = region
        .as_ref()
        .map(|r| Cow::Borrowed(r.borrow()))
        .unwrap_or_default();

    let mut request = SignedRequest::new("GET", "sts", &region, "/");

    let mut params = Params::new();
    params.put("Action", "GetCallerIdentity");
    params.put("Version", "2011-06-15");
    request.set_params(params);

    for (header, value) in additional_headers.into_iter() {
        request.add_header(header, value)
    }

    request.generate_presigned_url(credentials, expires_in.unwrap_or(&DEFAULT_EXPIRES), true)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use rusoto_core::credential::ProvideAwsCredentials;

    // mock_key, mock_secret
    pub(crate) async fn credentials() -> Result<AwsCredentials, crate::Error> {
        let provider = rusoto_mock::MockCredentialsProvider;
        Ok(provider.credentials().await?)
    }

    pub(crate) async fn post_aws_iam_payload(
        region: Option<Region>,
        header: HashMap<&str, &str>,
    ) -> Result<AwsAuthIamPayload, crate::Error> {
        let cred = credentials().await?;
        Ok(AwsAuthIamPayload::new(&cred, region, header))
    }

    pub(crate) async fn get_presigned_url(
        region: Option<Region>,
        header: HashMap<&str, &str>,
    ) -> Result<String, crate::Error> {
        let cred = credentials().await?;
        Ok(presigned_url(&cred, region, header, None))
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn post_aws_iam_payload_has_expected_values() -> Result<(), crate::Error> {
        let region = Region::UsEast1;
        let headers = [("X-Vault-AWS-IAM-Server-ID", "vault.example.com")]
            .iter()
            .cloned()
            .collect();
        let payload = post_aws_iam_payload(Some(region.clone()), headers).await?;

        assert_eq!(payload.iam_http_request_method, "POST");
        assert_eq!(
            payload.iam_request_url,
            base64::encode(&format!("https://sts.{}.amazonaws.com/", region.name()))
        );
        assert_eq!(
            payload.iam_request_body,
            base64::encode("Action=GetCallerIdentity&Version=2011-06-15")
        );
        assert!(payload.iam_request_headers.contains_key("authorization"));
        assert_eq!(
            payload
                .iam_request_headers
                .get(&"X-Vault-AWS-IAM-Server-ID".to_lowercase()),
            Some(&vec!["vault.example.com".to_string()])
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn presigned_url_has_expected_values() -> Result<(), crate::Error> {
        let region = Region::UsEast1;
        let headers = [("X-K8S-AWS-ID", "example")].iter().cloned().collect();
        let url = get_presigned_url(Some(region), headers).await?;
        let url = url::Url::parse(&url).unwrap();

        assert_eq!(
            url.host().unwrap().to_string(),
            "sts.us-east-1.amazonaws.com"
        );

        let params: HashMap<_, _> = url.query_pairs().collect();

        assert_eq!(params["Action"], "GetCallerIdentity");
        assert_eq!(params["Version"], "2011-06-15");
        assert_eq!(params["X-Amz-SignedHeaders"], "host;x-k8s-aws-id");
        assert_eq!(params["X-Amz-Expires"], "60");
        assert_eq!(params["X-Amz-Algorithm"], "AWS4-HMAC-SHA256");

        assert!(params.contains_key("X-Amz-Signature"));
        assert!(params.contains_key("X-Amz-Credential"));
        assert!(params.contains_key("X-Amz-Date"));
        Ok(())
    }
}

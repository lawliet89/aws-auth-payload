# `aws-auth-payload`

This library provides methods for you to use your AWS credentials to generate a pre-signed
request to AWS API.

You can then pass this pre-signed request to an appliaction you are using for the application
to verify your AWS identity.

This is what [Vault](https://www.vaultproject.io/docs/auth/aws.html#iam-auth-method) uses to
authenticate AWS users.

It is also used in [Kubernetes](https://github.com/kubernetes-sigs/aws-iam-authenticator) to
authenticate AWS IAM users.

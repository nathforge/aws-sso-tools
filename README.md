# AWS SSO Tools

Quality of life improvements for engineers using AWS SSO.

 * macOS: fully supported & tested.
 * Linux: untested. Does it work for you on Linux? Open an issue, let me know.
 * Windows: not yet supported.


## The tools

### aws-sso-maybe-login <a name="aws-sso-maybe-login"></a>

Perform an SSO login only when needed.


### aws-sso-login-showing-code <a name="aws-sso-login-showing-code"></a>

AWS SSO login CLI shows a security code - this displays it in a popup for easy verification.

> ![](docs/aws-sso-show-code.png)

(Full functionality on macOS only at present. The code window won't be displayed otherwise, but login will behave as normal.)


### docker-credential-sso-ecr-login <a name="docker-credential-sso-ecr-login"></a>

When interacting with an ECR repo this will perform an SSO login when needed and then login to ECR.

An extension of [aws-docker-credential-ecr-helper](https://github.com/awslabs/amazon-ecr-credential-helper) which must be installed separately.

See the [Docker install notes](https://github.com/awslabs/amazon-ecr-credential-helper#docker) on amazon-ecr-credential-helper. Where it adds ` "ecr-login"` to the Docker config, use `"sso-ecr-login"` instead. 


## Installation

Homebrew:
```shell
brew tap nathforge/tap
brew trust nathforge/tap
brew install aws-sso-tools
```

Mise:
```shell
MISE_MINIMUM_RELEASE_AGE=0s mise use github:nathforge/aws-sso-tools
```

# AWS SSO Tools

Enable automatic AWS SSO login.

**The goal: never run `aws sso login` again.**


## Installation

macOS: fully supported; Linux: untested.

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


## Recommended setup

 1. Add `alias aws="aws-sso-run aws"` to your shell startup script (e.g ~/.zshrc). `aws` commands now automatically login when needed.
 2. For services you're developing, run them with `aws-sso-run [...]` so they also login if needed at startup.
 3. Setup [docker-credential-sso-ecr-login](#docker-credential-sso-ecr-login) for automatic SSO/ECR login when using Docker commands.


## How does auto login work?

If the current profile uses SSO and the token has expired, it begins the login process.
Otherwise it'll continue as normal.

The login check is purely local so adds little overhead. The tools are written in Rust
for small size and fast startup.


## The main tools

### aws-sso-run

Uses `aws sso login` when needed, then runs the given command.

Examples:
 * `aws-sso-run ./start-my-service`
 * `aws-sso-run aws sts get-caller-identity`
 * `alias aws="aws-sso-run aws"`


### docker-credential-sso-ecr-login <a name="docker-credential-sso-ecr-login"></a>

Use `docker push`, `docker pull` etc. on ECR repos without worrying about auth.

Requires that [aws-docker-credential-ecr-helper](https://github.com/awslabs/amazon-ecr-credential-helper) is already installed.

To install see [these notes](https://github.com/awslabs/amazon-ecr-credential-helper#docker) - but where `"ecr-login"` is added to the Docker config, replace it with `"sso-ecr-login"`.


## The helpers

Helper binaries used by the above commands, and for you to write your own tools.

### aws-sso-login-showing-code

`aws sso login` displays a security code which you're supposed to verify in the web UI.

This command runs `aws sso login` and shows the code in a popup for easy verification.

> ![](docs/aws-sso-show-code.png)

*(Called by `aws-sso-maybe-login`)*

### aws-sso-show-code

Shows the above code popup.

*(Called by `aws-sso-login-showing-code`)*


### aws-sso-maybe-login <a name="aws-sso-maybe-login"></a>

Perform an SSO login only when needed.

*(Called by `aws-sso-run`, `docker-credential-sso-ecr-login`)*


### aws-sso-should-login

Checks if an SSO login is needed - returns code 0 when login is needed, otherwise 1.

Example:
```shell
if aws-sso-should-login; then
  # [...]
fi
```

*(Called by `aws-sso-maybe-login`)*

# aws-sso-tools

QoL improvements for engineers using AWS SSO.

## Install

### Homebrew

```
brew tap nathforge/tap
brew trust nathforge/tap
brew install aws-sso-tools
```

### mise

```
MISE_MINIMUM_RELEASE_AGE=0s mise use github:nathforge/aws-sso-tools
```

The env var overrides mise's 24h release age guard, which otherwise blocks newly published releases.

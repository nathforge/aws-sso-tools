#!/usr/bin/env bash
set -euo pipefail

BINARIES=(
    docker-credential-sso-ecr-login
    aws-sso-should-login
    aws-sso-maybe-login
    aws-sso-login-showing-code
    aws-sso-show-code
)

mkdir -p dist/staging/darwin_arm64 dist/staging/darwin_amd64

for b in "${BINARIES[@]}"; do
    cp "rust/target/aarch64-apple-darwin/release/$b" "dist/staging/darwin_arm64/$b"
    cp "rust/target/x86_64-apple-darwin/release/$b"  "dist/staging/darwin_amd64/$b"
done

tar -czf dist/aws-sso-tools_darwin_arm64.tar.gz -C dist/staging/darwin_arm64 "${BINARIES[@]}"
tar -czf dist/aws-sso-tools_darwin_amd64.tar.gz -C dist/staging/darwin_amd64 "${BINARIES[@]}"

(cd dist && shasum -a 256 aws-sso-tools_darwin_*.tar.gz > checksums.txt)

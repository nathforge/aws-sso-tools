#!/usr/bin/env bash
set -euo pipefail

BINARIES=(
    docker-credential-sso-ecr-login
    aws-sso-should-login
    aws-sso-maybe-login
    aws-sso-login-showing-code
    aws-sso-show-code
)

rustup target add aarch64-apple-darwin x86_64-apple-darwin

(cd rust && cargo build --release --target aarch64-apple-darwin)
(cd rust && cargo build --release --target x86_64-apple-darwin)

mkdir -p dist/prebuilt/darwin_arm64 dist/prebuilt/darwin_amd64

for b in "${BINARIES[@]}"; do
    cp "rust/target/aarch64-apple-darwin/release/$b" "dist/prebuilt/darwin_arm64/$b"
    cp "rust/target/x86_64-apple-darwin/release/$b" "dist/prebuilt/darwin_amd64/$b"
done

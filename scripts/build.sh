#!/usr/bin/env bash
set -euo pipefail

EXTRA_BINARIES=(
    aws-sso-should-login
    aws-sso-maybe-login
    aws-sso-login-showing-code
    aws-sso-show-code
)

rustup target add aarch64-apple-darwin x86_64-apple-darwin

(cd rust && cargo build --release --target aarch64-apple-darwin)
(cd rust && cargo build --release --target x86_64-apple-darwin)

mkdir -p dist/extras/darwin_arm64 dist/extras/darwin_amd64

for b in "${EXTRA_BINARIES[@]}"; do
    cp "rust/target/aarch64-apple-darwin/release/$b" "dist/extras/darwin_arm64/$b"
    cp "rust/target/x86_64-apple-darwin/release/$b" "dist/extras/darwin_amd64/$b"
done

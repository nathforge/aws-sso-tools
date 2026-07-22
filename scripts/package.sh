#!/usr/bin/env bash
set -euo pipefail

BINARIES=(
    docker-credential-sso-ecr-login
    aws-sso-should-login
    aws-sso-maybe-login
    aws-sso-login-showing-code
    aws-sso-show-code
)

mkdir -p dist

case "$(uname -s)" in
  Darwin)
    STAGING=$(mktemp -d)
    mkdir -p "$STAGING/darwin_arm64" "$STAGING/darwin_amd64"
    for b in "${BINARIES[@]}"; do
        cp "target/aarch64-apple-darwin/release/$b" "$STAGING/darwin_arm64/$b"
        cp "target/x86_64-apple-darwin/release/$b"  "$STAGING/darwin_amd64/$b"
    done
    tar -czf dist/aws-sso-tools_darwin_arm64.tar.gz -C "$STAGING/darwin_arm64" "${BINARIES[@]}"
    tar -czf dist/aws-sso-tools_darwin_amd64.tar.gz -C "$STAGING/darwin_amd64" "${BINARIES[@]}"
    ;;
  Linux)
    STAGING=$(mktemp -d)
    mkdir -p "$STAGING/linux_amd64" "$STAGING/linux_arm64"
    for b in "${BINARIES[@]}"; do
        cp "target/x86_64-unknown-linux-gnu/release/$b"  "$STAGING/linux_amd64/$b"
        cp "target/aarch64-unknown-linux-gnu/release/$b" "$STAGING/linux_arm64/$b"
    done
    tar -czf dist/aws-sso-tools_linux_amd64.tar.gz -C "$STAGING/linux_amd64" "${BINARIES[@]}"
    tar -czf dist/aws-sso-tools_linux_arm64.tar.gz -C "$STAGING/linux_arm64" "${BINARIES[@]}"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    STAGING=$(mktemp -d)
    for b in "${BINARIES[@]}"; do
        cp "target/x86_64-pc-windows-msvc/release/${b}.exe" "$STAGING/${b}.exe"
    done
    7z a dist/aws-sso-tools_windows_amd64.zip "$STAGING"/*.exe
    ;;
  *)
    echo "Unsupported platform: $(uname -s)" >&2
    exit 1
    ;;
esac

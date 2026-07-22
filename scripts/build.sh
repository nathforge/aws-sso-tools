#!/usr/bin/env bash
set -euo pipefail

rustup target add aarch64-apple-darwin x86_64-apple-darwin
(cd rust && cargo build --release --target aarch64-apple-darwin)
(cd rust && cargo build --release --target x86_64-apple-darwin)

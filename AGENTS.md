## Build & test

```bash
cargo build           # debug
cargo build --release # release
cargo test            # unit tests
```

Cross-compilation for all four platform targets is handled by `scripts/build.sh`.

## Release process

Ensure you're on `main` with a clean working tree, then:

```bash
./scripts/release.sh major|minor|patch
```

This will:
1. Bump the version in `Cargo.toml` / `Cargo.lock`
2. Run `cargo check`
3. Commit `Release vX.Y.Z`, tag `vX.Y.Z`, push `main` + the tag
4. Watch the `release.yml` GitHub Actions workflow to completion

CI then builds all platform targets, creates the GitHub release, and updates the Homebrew tap at `nathforge/tap`.

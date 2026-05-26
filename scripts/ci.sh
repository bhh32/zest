#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all --check
cargo check --workspace --quiet
cargo check -p zest-widget --examples --quiet
cargo check -p zest-gui --features simulator --quiet
cargo doc --workspace --no-deps

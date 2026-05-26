#!/usr/bin/env bash
set -euo pipefail

cargo publish --locked -p zest-theme
sleep 60
cargo publish --locked -p zest-core
sleep 60
cargo publish --locked -p zest-widget
sleep 60
cargo publish --locked -p zest-simulator
sleep 60
cargo publish --locked -p zest-gui

#!/usr/bin/env bash
# Full release build. Each module's Trunk.toml already points its output at
# assets/, so these are plain trunk builds — the only thing that matters is
# the order: copy-dir needs assets/{game,life} populated before the site
# build runs.
#
# Day to day you don't need this. Use `trunk serve` for site work, and only
# rebuild a module when you've changed web-fps or life-v2.
set -euo pipefail
cd "$(dirname "$0")"

(cd crates/life-wasm && trunk build --release)
(cd crates/game-wasm && trunk build --release)
trunk build --release

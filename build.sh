#!/bin/sh
set -eu
cd "$(dirname "$0")"
python3 tools/package-web.py
cargo build --locked --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/raccoon_engine.wasm dist/port/ppk.wasm
chmod 644 dist/port/ppk.wasm
python3 tools/build-service-worker.py

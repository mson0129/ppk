#!/bin/sh
set -eu
cd "$(dirname "$0")"
python3 tools/rust-assets.py
python3 tools/package-web.py
(cd rust && cargo build --locked --release --target wasm32-unknown-unknown)
cp rust/target/wasm32-unknown-unknown/release/raccoon_engine.wasm dist/port/ppk.wasm
chmod 644 dist/port/ppk.wasm
python3 tools/build-service-worker.py

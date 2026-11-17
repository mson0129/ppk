#!/bin/sh
set -eu
cd "$(dirname "$0")"
: "${WASI_SDK_PATH:?Set WASI_SDK_PATH to your WASI SDK directory}"
python3 tools/assets.py
python3 tools/package-web.py
"$WASI_SDK_PATH/bin/clang" -O2 -nostartfiles -Wno-parentheses-equality \
  -Wl,--no-entry -Wl,--export-dynamic -Wl,--export-memory \
  -Wl,-z,stack-size=262144 -Wl,--initial-memory=2097152 -Wl,--max-memory=2097152 \
  src/port.c -o dist/port/ppk.wasm
chmod 644 dist/port/ppk.wasm
python3 tools/build-bundle.py

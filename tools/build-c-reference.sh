#!/bin/sh
# Frozen C port as an independent differential oracle; never writes the web build.
set -eu
cd "$(dirname "$0")/.."
: "${WASI_SDK_PATH:?Set WASI_SDK_PATH to your WASI SDK directory}"
"$WASI_SDK_PATH/bin/clang" -O2 -nostartfiles -Wno-parentheses-equality \
 -Wl,--no-entry -Wl,--export-dynamic -Wl,--export-memory \
 -Wl,-z,stack-size=262144 -Wl,--initial-memory=2097152 -Wl,--max-memory=2097152 \
 src/port.c -o analysis/c-reference.wasm

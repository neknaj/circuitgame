#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

CARGO_HOME="${CARGO_HOME:-/tmp/cargo-home}"
mkdir -p "$CARGO_HOME" ./dist

if command -v wasm-bindgen >/dev/null 2>&1; then
    WASM_BINDGEN_BIN="$(command -v wasm-bindgen)"
elif [ -x "${HOME:-}/.cargo/bin/wasm-bindgen" ]; then
    WASM_BINDGEN_BIN="${HOME}/.cargo/bin/wasm-bindgen"
else
    echo "wasm-bindgen not found. Install it with: cargo install wasm-bindgen-cli" >&2
    exit 1
fi

export CARGO_HOME

cargo build --lib --release --target wasm32-unknown-unknown --no-default-features --features egui-web
"$WASM_BINDGEN_BIN" --target web --no-typescript --out-dir ./dist ./target/wasm32-unknown-unknown/release/circuitgame_lib.wasm

cp ./src/web/index.html ./dist/index.html
cp ./spec/icon.ico ./dist/favicon.ico
cp ./spec/icon.png ./dist/icon.png

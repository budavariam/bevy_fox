#!/bin/sh
export PATH="$HOME/.cargo/bin:$PATH"

set -e

rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.100
cargo build --release --target wasm32-unknown-unknown

wasm-bindgen --out-dir out/ --target web target/wasm32-unknown-unknown/release/bevy_fox.wasm

cp -R assets out/ 2>/dev/null || true
cp static/index.html out/index.html

touch out/.nojekyll
#!/bin/bash

export PATH="$HOME/.cargo/bin:$PATH"

cargo install cargo-watch
cargo watch -x run
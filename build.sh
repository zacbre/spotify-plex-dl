#!/usr/bin/env zsh

cargo-zigbuild build --release --target x86_64-unknown-linux-musl || exit 1
cargo-zigbuild build --release --target x86_64-pc-windows-gnu || exit 1
cargo build --release || exit 1

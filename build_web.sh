#!/usr/bin/env bash

cargo install wasm-bindgen-cli
rustup target add wasm32-unknown-unknown
cargo test --workspace
(cd web/employee-scheduling-wasm-bindgen && cargo build --target wasm32-unknown-unknown --profile production)
(cd web/employee-scheduling && npm run build)

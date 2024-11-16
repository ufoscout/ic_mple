#!/usr/bin/env bash
set -e
set -x
export RUST_BACKTRACE=full

cargo test 
cargo test --all-features

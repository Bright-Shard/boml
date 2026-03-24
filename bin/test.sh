#!/bin/sh

ROOT=$(dirname $0)/..
MANIFEST=$ROOT/Cargo.toml

cargo test --manifest-path $MANIFEST
cargo test --no-default-features $MANIFEST
cargo test --all-features $MANIFEST
RUSTDOCFLAGS="-D warnings" cargo doc --all-features

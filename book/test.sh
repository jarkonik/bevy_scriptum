#!/bin/sh

CARGO_MANIFEST_DIR=$(pwd) cargo clean && cargo build && mdbook test -L target/debug/deps/

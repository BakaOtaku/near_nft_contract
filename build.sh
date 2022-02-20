#!/bin/bash
set -e
cd "`dirname $0`"
source flags.sh
#cargo rustc --Awarnings
cargo build --all  --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
#!/bin/bash

set -e

cargo clean

RUSTFLAGS="-Cinstrument-coverage" cargo test

grcov . -s . --binary-path ./target/debug/ -t html --ignore tests/  -o ./target/debug/coverage/

rm *.profraw

open target/debug/coverage/index.html

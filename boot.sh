#!/bin/bash

git pull origin master

RUST_BACKTRACE=full RUST_LOG=info cargo run --release --example main -- $@

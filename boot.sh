#!/bin/bash

git pull origin master

RUST_BACKTRACE=1 RUST_LOG=main cargo run --release --example main -- $@

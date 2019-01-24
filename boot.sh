#!/bin/bash

git pull origin master

RUST_BACKTRACE=1 RUST_LOG=main,examplehttp cargo run --release --example main -- $@

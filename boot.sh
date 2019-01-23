#!/bin/bash

PORT=$1
PORT=${PORT:-9000}

git pull origin master

RUST_BACKTRACE=1 cargo run --release --example main -- --port=${PORT}

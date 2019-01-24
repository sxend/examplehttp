#!/bin/bash

PORT=$1
PORT=${PORT:-8888}
USE_LOOP=$2
USE_LOOP=${USE_LOOP:-true}

git pull origin master

RUST_BACKTRACE=1 cargo run --release --example main -- --port=${PORT} --use-loop=${USE_LOOP}

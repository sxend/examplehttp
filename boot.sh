#!/bin/bash

PORT=$1
PORT=${PORT:-9000}

git pull origin master

cargo run --release --example main -- --port=${PORT}

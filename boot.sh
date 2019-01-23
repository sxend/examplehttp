#!/bin/bash

PORT=$1
PORT=${PORT:-80}

git pull origin master

 cargo run --release --example main -- --port=$PORT

#!/bin/bash
cd "$(dirname "$0")/corpus-server"
echo "Starting Corpus+ backend server..."
cargo run --release

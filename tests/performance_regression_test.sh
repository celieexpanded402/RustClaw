#!/bin/bash
# Performance Regression Test Script
# Track build time and binary size
current_dir=$(pwd)
build_start_time=$(date +%s)
cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed"
    exit 1
fi
build_end_time=$(date +%s)
binary_size=$(stat -c %s target/release/myapp)
echo "Build time: $((build_end_time-build_start_time)) seconds"
echo "Binary size: $binary_size bytes"
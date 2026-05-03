#!/bin/bash

# Run all tests with single thread to avoid race conditions
cargo test --test integration -- --test-threads=1
cargo test --lib -- --test-threads=1
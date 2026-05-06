#!/bin/bash
set -x

echo "=== Building git5 ==="
cargo build
if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "=== Running integration tests (single threaded) ==="
cargo test --test integration -- --test-threads=1
if [ $? -ne 0 ]; then
    echo "Integration tests failed!"
    exit 1
fi

echo ""
echo "=== Running library tests (single threaded) ==="
cargo test --lib -- --test-threads=1
if [ $? -ne 0 ]; then
    echo "Library tests failed!"
    exit 1
fi

echo ""
echo "=== All tests passed! ==="
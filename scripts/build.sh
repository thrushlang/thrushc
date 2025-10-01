#!/bin/bash
# Build and package script for thrushc
# Usage: ./build-release.sh [debug|release]
# This script should be run from the /scripts directory

set -e

build_mode="release"
project_root=".."
target_dir="$project_root/target"
binary_name="thrushc"
dist_dir="$project_root/dist/linux"

if [ $# -gt 0 ]; then
    build_mode="$1"
fi

if [[ "$build_mode" != "debug" && "$build_mode" != "release" ]]; then
    echo "Error: Invalid build mode '$build_mode'. Use 'debug' or 'release'."
    exit 1
fi

if [ ! -f "$project_root/Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Make sure this script is run from the /scripts directory."
    exit 1
fi

if ! command -v upx &> /dev/null; then
    echo "Error: UPX not found. Please install UPX for binary compression."
    exit 1
fi

if [ "$build_mode" = "release" ] && ! command -v strip &> /dev/null; then
    echo "Warning: strip command not found. Binary will not be stripped."
fi

echo "Building project in $build_mode mode..."

pushd "$project_root" > /dev/null

if [ "$build_mode" = "release" ]; then
    cargo build --release
    binary_path="target/release/$binary_name"
else
    cargo build
    binary_path="target/debug/$binary_name"
fi

build_success=$?
popd > /dev/null

if [ $build_success -ne 0 ]; then
    echo "Error: Build failed."
    exit 1
fi

if [ "$build_mode" = "release" ]; then
    source_binary="$target_dir/release/$binary_name"
else
    source_binary="$target_dir/debug/$binary_name"
fi

if [ ! -f "$source_binary" ]; then
    echo "Error: Binary '$binary_name' not found at $source_binary"
    exit 1
fi

echo "Build completed successfully."

mkdir -p "$dist_dir"
dist_binary="$dist_dir/$binary_name"
cp "$source_binary" "$dist_binary"

if [ "$build_mode" = "release" ] && command -v strip &> /dev/null; then
    echo "Stripping binary..."
    if strip "$dist_binary"; then
        echo "Binary stripped successfully."
    else
        echo "Warning: Failed to strip binary."
    fi
fi

echo "Compressing binary with UPX..."
if upx --best "$dist_binary"; then
    echo "Binary compressed successfully."
    final_size=$(du -h "$dist_binary" | cut -f1)
    echo "Final binary: $dist_binary ($final_size)"
    echo "Build and packaging completed."
else
    echo "Error: UPX compression failed."
    exit 1
fi
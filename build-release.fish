#!/usr/bin/env fish

# Build and package script for thrushc
# Usage: ./build-release.fish [debug|release]

set -l build_mode "release"
set -l target_dir "target"
set -l binary_name "thrushc"
set -l dist_dir "dist/linux"

if test (count $argv) -gt 0
    set build_mode $argv[1]
end

if not contains $build_mode "debug" "release"
    echo "Error: Invalid build mode '$build_mode'. Use 'debug' or 'release'."
    exit 1
end

if not test -f "Cargo.toml"
    echo "Error: Cargo.toml not found. Run this script from the project root."
    exit 1
end

if not command -q upx
    echo "Error: UPX not found. Please install UPX for binary compression."
    exit 1
end

if test "$build_mode" = "release"
    if not command -q strip
        echo "Warning: strip command not found. Binary will not be stripped."
    end
end

echo "Building project in $build_mode mode..."

if test "$build_mode" = "release"
    cargo build --release
    set binary_path "$target_dir/release/$binary_name"
else
    cargo build
    set binary_path "$target_dir/debug/$binary_name"
end

if test $status -ne 0
    echo "Error: Build failed."
    exit 1
end

if not test -f "$binary_path"
    echo "Error: Binary '$binary_name' not found at $binary_path"
    exit 1
end

echo "Build completed successfully."

mkdir -p "$dist_dir"

set dist_binary "$dist_dir/$binary_name"
cp "$binary_path" "$dist_binary"

if test "$build_mode" = "release"; and command -q strip
    echo "Stripping binary..."
    strip "$dist_binary"
    if test $status -eq 0
        echo "Binary stripped successfully."
    else
        echo "Warning: Failed to strip binary."
    end
end

echo "Compressing binary with UPX..."
upx --best "$dist_binary"

if test $status -eq 0
    echo "Binary compressed successfully."
    
    set final_size (du -h "$dist_binary" | cut -f1)
    echo "Final binary: $dist_binary ($final_size)"
    echo "Build and packaging completed."
else
    echo "Error: UPX compression failed."
    exit 1
end
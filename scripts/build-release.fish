#!/usr/bin/env fish
# Build and package script for thrushc
# Usage: ./build-release.fish [debug|release]
# This script should be run from the /scripts directory

set -l build_mode "release"
set -l project_root ".."
set -l target_dir "$project_root/target"
set -l binary_name "thrushc"
set -l dist_dir "$project_root/dist/linux"

if test (count $argv) -gt 0
    set build_mode $argv[1]
end

if not contains $build_mode "debug" "release"
    echo "Error: Invalid build mode '$build_mode'. Use 'debug' or 'release'."
    exit 1
end

if not test -f "$project_root/Cargo.toml"
    echo "Error: Cargo.toml not found. Make sure this script is run from the /scripts directory."
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

# Change to project root directory for cargo build
pushd "$project_root"

if test "$build_mode" = "release"
    cargo build --release
    set binary_path "target/release/$binary_name"
else
    cargo build
    set binary_path "target/debug/$binary_name"
end

set build_result $status
popd

if test $build_result -ne 0
    echo "Error: Build failed."
    exit 1
end

if test "$build_mode" = "release"
    set source_binary "$target_dir/release/$binary_name"
else
    set source_binary "$target_dir/debug/$binary_name"
end

if not test -f "$source_binary"
    echo "Error: Binary '$binary_name' not found at $source_binary"
    exit 1
end

echo "Build completed successfully."

mkdir -p "$dist_dir"
set dist_binary "$dist_dir/$binary_name"
cp "$source_binary" "$dist_binary"

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
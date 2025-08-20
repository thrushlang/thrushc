# Usage: .\build-release.ps1 [debug|release]

$ErrorActionPreference = "Stop"

$build_mode = "release"
$project_root = ".."
$target_dir = Join-Path $project_root "target"
$binary_name = "thrushc"
$dist_dir = Join-Path $project_root "dist/linux"

if ($args.Count -gt 0) {
    $build_mode = $args[0]
}

if ($build_mode -ne "debug" -and $build_mode -ne "release") {
    Write-Error "Invalid build mode '$build_mode'. Use 'debug' or 'release'."
    exit 1
}

$cargo_toml = Join-Path $project_root "Cargo.toml"

if (-not (Test-Path $cargo_toml)) {
    Write-Error "Cargo.toml not found. Make sure this script is run from the scripts directory."
    exit 1
}

if (-not (Get-Command upx -ErrorAction SilentlyContinue)) {
    Write-Error "UPX not found. Please install UPX for binary compression."
    exit 1
}

Write-Host "Building project in $build_mode mode..."

Push-Location $project_root

try {
    if ($build_mode -eq "release") {
        cargo build --release
        $binary_path = Join-Path "target/release" $binary_name
    } else {
        cargo build
        $binary_path = Join-Path "target/debug" $binary_name
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed."
        exit 1
    }
}
finally {
    Pop-Location
}

if ($build_mode -eq "release") {
    $source_binary = Join-Path $target_dir "release/$binary_name"
} else {
    $source_binary = Join-Path $target_dir "debug/$binary_name"
}

if (-not (Test-Path $source_binary)) {
    Write-Error "Binary '$binary_name' not found at $source_binary"
    exit 1
}

Write-Host "Build completed successfully."

New-Item -ItemType Directory -Force -Path $dist_dir | Out-Null
$dist_binary = Join-Path $dist_dir $binary_name
Copy-Item -Path $source_binary -Destination $dist_binary -Force

Write-Host "Compressing binary with UPX..."
$upxResult = & upx --best $dist_binary 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "Binary compressed successfully."
    $final_size = (Get-Item $dist_binary).Length / 1MB
    $final_size = "{0:N2} MB" -f $final_size
    Write-Host "Final binary: $dist_binary ($final_size)"
    Write-Host "Build and packaging completed."
} else {
    Write-Error "UPX compression failed: $upxResult"
    exit 1
}
if (-not (Get-Command cross -ErrorAction SilentlyContinue)) {
    cargo install cross
}
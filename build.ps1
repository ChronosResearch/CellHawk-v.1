<#
.SYNOPSIS
Enterprise Build Script for CELLHAWK

.DESCRIPTION
This script handles the full build lifecycle including linting, testing, and 
release compilation for the CELLHAWK workspace. It requires the MSVC toolchain.

.EXAMPLE
.\build.ps1 -Clean -Test
#>

param(
    [switch]$Clean,
    [switch]$Test
)

$ErrorActionPreference = "Stop"

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host " CELLHAWK Enterprise Build Pipeline" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

if ($Clean) {
    Write-Host "[+] Cleaning workspace..." -ForegroundColor Yellow
    cargo clean
}

Write-Host "[+] Checking code formatting..." -ForegroundColor Yellow
cargo fmt --all -- --check

Write-Host "[+] Running strict Clippy lints (Deny unwrap/unsafe)..." -ForegroundColor Yellow
cargo clippy --workspace --all-targets --all-features -- -D warnings

if ($Test) {
    Write-Host "[+] Running mathematical verification suites..." -ForegroundColor Yellow
    cargo test --workspace --all-features
}

Write-Host "[+] Compiling release binaries..." -ForegroundColor Yellow
cargo build --release --workspace

Write-Host "=========================================" -ForegroundColor Green
Write-Host " Build Complete! Binaries in /target/release/" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green

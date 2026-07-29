# CELLHAWK Edge Watchdog Supervisor
# This script ensures the drone control node stays alive. If it crashes, it automatically restarts it.
Write-Host "CELLHAWK PRODUCTION WATCHDOG STARTED" -ForegroundColor Cyan

$maxRetries = 10
$retryCount = 0
$exePath = ".\target\release\cellhawk-edge.exe"

# If release build doesn't exist, fallback to cargo run (development mode)
$useCargo = -not (Test-Path $exePath)
if ($useCargo) {
    Write-Host "Release binary not found. Falling back to 'cargo run --release --bin cellhawk-edge'" -ForegroundColor Yellow
}

while ($retryCount -lt $maxRetries) {
    Write-Host "Starting CELLHAWK Node... (Attempt $($retryCount + 1))" -ForegroundColor Green
    
    if ($useCargo) {
        $process = Start-Process -FilePath "cargo" -ArgumentList "run", "--release", "--bin", "cellhawk-edge" -Wait -PassThru -NoNewWindow
    } else {
        $process = Start-Process -FilePath $exePath -Wait -PassThru -NoNewWindow
    }
    
    if ($process.ExitCode -eq 0) {
        Write-Host "CELLHAWK shut down gracefully." -ForegroundColor Green
        break
    } else {
        $retryCount++
        Write-Host "CRITICAL: CELLHAWK CRASHED! (Exit code: $($process.ExitCode))" -ForegroundColor Red
        Write-Host "Watchdog initiating auto-recovery in 3 seconds..." -ForegroundColor Yellow
        Start-Sleep -Seconds 3
    }
}

if ($retryCount -ge $maxRetries) {
    Write-Host "FATAL: CELLHAWK failed to recover after $maxRetries attempts. Halting." -ForegroundColor Red
}

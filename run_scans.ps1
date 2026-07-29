$ErrorActionPreference = "Continue"

Write-Output "Running STEP 2: Vulnerability Scans..." | Out-File "D:\CELLHAWK\SCAN_REPORT.txt"

Write-Output "`n--- Rust (cargo deny check advisories) ---" | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"
Set-Location "D:\CELLHAWK"
cargo deny check advisories 2>&1 | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"

Write-Output "`n--- Rust (cargo audit) ---" | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"
cargo audit 2>&1 | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"

Write-Output "`n--- JS (npm audit) ---" | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"
Set-Location "D:\CELLHAWK\gcs-ui"
cmd /c "npm audit" 2>&1 | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"

Write-Output "`n--- Python (bandit) ---" | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"
Set-Location "D:\CELLHAWK"
python -m bandit -r .\crates\cellhawk-gcs 2>&1 | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"

Write-Output "`n--- C++ (cve-bin-tool) ---" | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"
python -m cve_bin_tool.cli . 2>&1 | Out-File -Append "D:\CELLHAWK\SCAN_REPORT.txt"

Write-Output "Scans Complete."

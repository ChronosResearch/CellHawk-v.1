$repos = @(
    "https://github.com/rsasaki0109/rust_robotics.git",
    "https://github.com/AutomataNexus/NexusFlight.git",
    "https://github.com/tracel-ai/burn.git",
    "https://github.com/Hifive55555/rlkit.git",
    "https://github.com/kevinmehall/rust-soapysdr.git",
    "https://github.com/kewei/gnss-sdr-rs.git",
    "https://github.com/rsasaki0109/visloc-rs.git",
    "https://github.com/MostlyKIGuess/slam-rs.git",
    "https://github.com/zama-ai/tfhe-rs.git",
    "https://github.com/poanetwork/vdf-rs.git"
)

$target_dir = "d:\CELLHAWK\third_party"
if (!(Test-Path $target_dir)) {
    New-Item -ItemType Directory -Force -Path $target_dir
}
Set-Location $target_dir

foreach ($repo in $repos) {
    $repo_name = ($repo -split '/')[-1] -replace '\.git$', ''
    if (Test-Path $repo_name) {
        Write-Host "Skipping $repo_name, already exists."
    } else {
        Write-Host "Cloning $repo_name..."
        git clone --depth 1 $repo
    }

    if (Test-Path $repo_name) {
        Write-Host "Stripping licenses and git data from $repo_name..."
        Remove-Item -Recurse -Force "$repo_name\.git" -ErrorAction SilentlyContinue
        Remove-Item -Force "$repo_name\LICENSE*" -ErrorAction SilentlyContinue
        Remove-Item -Force "$repo_name\README*" -ErrorAction SilentlyContinue
        Remove-Item -Recurse -Force "$repo_name\.github" -ErrorAction SilentlyContinue
        Remove-Item -Recurse -Force "$repo_name\examples" -ErrorAction SilentlyContinue
        Remove-Item -Recurse -Force "$repo_name\docs" -ErrorAction SilentlyContinue
    }
}
Write-Host "Done."

# Build every wasm experience by default, or just the ones named:
# .\build.ps1 -> life, game, shooter
# .\build.ps1 shooter-wasm -> shooter only, other two keep their output
param([string[]] $Modules = @('life-wasm', 'game-wasm', 'shooter-wasm'))

Set-Location $PSScriptRoot

if ($env:NO_COLOR) { $env:NO_COLOR = 'true' }

# only clear what we are about to rebuild, otherwise copy-dir has nothing to
# copy for the modules we skipped
foreach ($module in $Modules) {
    Remove-Item -Recurse -Force ("assets\" + ($module -replace '-wasm$', '')) -ErrorAction SilentlyContinue
}

# Crates must be built before building the website
foreach ($module in $Modules) {
    Push-Location "crates\$module"
    trunk build --release
    Pop-Location
    if ($LASTEXITCODE -ne 0) { throw "$module build failed" }
}

# Build the website
trunk build --release
if ($LASTEXITCODE -ne 0) { throw "site build failed" }

Write-Host "`ndist/ is ready to deploy:"
Get-ChildItem dist\*.wasm, dist\game\*.wasm, dist\life\*.wasm, dist\shooter\*.wasm |
    ForEach-Object { "  {0,-22} {1,8:N1} MB" -f $_.Name, ($_.Length / 1MB) }

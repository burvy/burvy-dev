Set-Location $PSScriptRoot

if ($env:NO_COLOR) { $env:NO_COLOR = 'true' }

Remove-Item -Recurse -Force assets\game, assets\life -ErrorAction SilentlyContinue

# Crates must be built before building the website
foreach ($module in 'life-wasm', 'game-wasm') {
    Push-Location "crates\$module"
    trunk build --release
    Pop-Location
    if ($LASTEXITCODE -ne 0) { throw "$module build failed" }
}

# Build the website
trunk build --release
if ($LASTEXITCODE -ne 0) { throw "site build failed" }

Write-Host "`ndist/ is ready to deploy:"
Get-ChildItem dist\*.wasm, dist\game\*.wasm, dist\life\*.wasm |
    ForEach-Object { "  {0,-22} {1,8:N1} MB" -f $_.Name, ($_.Length / 1MB) }

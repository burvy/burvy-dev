# Build every wasm experience by default, or just the ones named:
# .\build.ps1 -> life, game, shooter, floret, venture (site only)
# .\build.ps1 shooter-wasm -> shooter only, other two keep their output
# .\build.ps1 -Deploy -> builds EVERYTHING (all wasm modules + both server
#                        executables), times each step, and gathers a
#                        deploy\ folder ready to copy to the server PC
param(
    [string[]] $Modules = @('life-wasm', 'game-wasm', 'shooter-wasm', 'floret-wasm', 'venture-wasm'),
    [switch] $Deploy
)

Set-Location $PSScriptRoot

if ($env:NO_COLOR) { $env:NO_COLOR = 'true' }

if ($Deploy) {
    # a full deploy always rebuilds every wasm module, not just the ones named
    $Modules = @('life-wasm', 'game-wasm', 'shooter-wasm', 'floret-wasm', 'venture-wasm')
}

$timings = [ordered]@{}
function Time-Step {
    param([string] $Name, [scriptblock] $Body)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $Body
    $sw.Stop()
    $timings[$Name] = $sw.Elapsed
}

$overall = [System.Diagnostics.Stopwatch]::StartNew()

# only clear what we are about to rebuild, otherwise copy-dir has nothing to
# copy for the modules we skipped
foreach ($module in $Modules) {
    Remove-Item -Recurse -Force ("assets\" + ($module -replace '-wasm$', '')) -ErrorAction SilentlyContinue
}

# Crates must be built before building the website
foreach ($module in $Modules) {
    Time-Step $module {
        Push-Location "crates\$module"
        trunk build --release
        Pop-Location
        if ($LASTEXITCODE -ne 0) { throw "$module build failed" }
    }
}

# Build the website
Time-Step 'site' {
    trunk build --release
    if ($LASTEXITCODE -ne 0) { throw "site build failed" }
}

if ($Deploy) {
    Time-Step 'burvy-dev/server' {
        Push-Location 'server'
        cargo build --release
        Pop-Location
        if ($LASTEXITCODE -ne 0) { throw "server build failed" }
    }

    Time-Step 'web-fps/game-server' {
        Push-Location '..\web-fps\game-server'
        cargo build --release
        Pop-Location
        if ($LASTEXITCODE -ne 0) { throw "game-server build failed" }
    }
}

$overall.Stop()

Write-Host "`nBuild times:"
foreach ($key in $timings.Keys) {
    Write-Host ("  {0,-20} {1,8:N1}s" -f $key, $timings[$key].TotalSeconds)
}
Write-Host ("  {0,-20} {1,8:N1}s" -f 'TOTAL', $overall.Elapsed.TotalSeconds)

Write-Host "`ndist/ is ready to deploy:"
Get-ChildItem dist\*.wasm, dist\game\*.wasm, dist\life\*.wasm, dist\shooter\*.wasm, dist\floret\*.wasm, dist\venture\*.wasm -ErrorAction SilentlyContinue |
    ForEach-Object { "  {0,-22} {1,8:N1} MB" -f $_.Name, ($_.Length / 1MB) }

if ($Deploy) {
    $deployDir = "deploy"
    Remove-Item -Recurse -Force $deployDir -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Path "$deployDir\site" -Force | Out-Null

    Copy-Item -Recurse -Force "dist\*" "$deployDir\site\"
    Copy-Item -Force "server\target\release\server.exe" "$deployDir\server.exe"
    Copy-Item -Force "..\web-fps\game-server\target\release\game-server.exe" "$deployDir\game-server.exe"

    Write-Host "`nDeploy folder ready at $deployDir\"
    Write-Host "Note: server.exe expects certs at C:\burvy\certs\webtrans.burvy.dev\ on the target machine - not included here, on purpose."

    Invoke-Item $deployDir
}

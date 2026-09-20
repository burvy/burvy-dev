# Build every wasm experience by default, or just the ones named:
# .\build.ps1                -> life, game, shooter, floret, venture (site only)
# .\build.ps1 shooter-wasm   -> shooter only, other experiences keep their output
# .\build.ps1 -Deploy        -> everything: all wasm modules + site + burvy-dev's own
#                               server.exe + a RELEASE build of every linked project's
#                               server (shooter-server, floret-server, game-server from
#                               web-fps), timed, all gathered into one deploy\ folder
#                               ready to copy to the server machine
# .\build.ps1 -Dev           -> serves the site on http://localhost:8080 and gathers
#                               every linked project's DEV-build server executable
#                               (each project's own DevArgs, e.g. --features dev-local -
#                               see $LinkedServers) into dev-servers\, so you can run
#                               whichever one you're testing against the live site
#
# Each linked project's own go.ps1 -release still works standalone (useful for
# redeploying just that one game without touching the rest of the site) - -Deploy
# here just builds the same release binaries itself so everything lands in one place.
param(
    [string[]] $Modules = @('life-wasm', 'game-wasm', 'shooter-wasm', 'floret-wasm', 'venture-wasm'),
    [string[]] $Servers = @('burvy-game', 'floret', 'web-fps'),
    [switch] $Deploy,
    [switch] $Dev
)

Set-Location $PSScriptRoot

if ($env:NO_COLOR) { $env:NO_COLOR = 'true' }

# Every linked project with its own multiplayer server. DevArgs is whatever extra
# cargo flags that project's own dev build needs (e.g. a dev-local feature) - release
# builds always just use --release, no per-project flags needed there.
# Add new entries here as new linked projects grow a server of their own.
$LinkedServers = @(
    @{ Project = 'burvy-game'; Package = 'shooter-server'; DevArgs = @('--features', 'dev-local') },
    @{ Project = 'floret';     Package = 'floret-server';  DevArgs = @('--features', 'dev-local') },
    @{ Project = 'web-fps';    Package = 'game-server';    DevArgs = @() }
)

if ($Deploy -or $Dev) {
    # a full build always rebuilds every wasm module, not just the ones named
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

# Builds every linked project's server package (respecting -Servers) and copies
# the resulting exe into $DestDir. -Release builds all of them with --release;
# otherwise each uses its own DevArgs (see $LinkedServers above).
function Build-LinkedServers {
    param([string] $DestDir, [switch] $Release, [switch] $Time)
    $profileDir = if ($Release) { 'release' } else { 'debug' }
    foreach ($linked in $LinkedServers) {
        if ($Servers -notcontains $linked.Project) { continue }

        $projectDir = Join-Path (Split-Path $PSScriptRoot -Parent) $linked.Project
        if (-not (Test-Path $projectDir)) {
            Write-Host "  skipping $($linked.Package): $projectDir not found" -ForegroundColor Yellow
            continue
        }

        # wrapped in @(...) so a single-element result stays an array instead of
        # collapsing to a bare string - splatting a scalar string enumerates it
        # character by character, which cargo sees as a run of single-dash args
        $cargoArgs = @(if ($Release) { '--release' } else { $linked.DevArgs })

        Write-Host "`n==> building $($linked.Package) ($profileDir)" -ForegroundColor Cyan
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        Push-Location $projectDir
        cargo build -p $linked.Package @cargoArgs
        $ok = ($LASTEXITCODE -eq 0)
        $targetDir = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).target_directory
        Pop-Location
        $sw.Stop()
        if ($Time) { $timings[$linked.Package] = $sw.Elapsed }
        if (-not $ok) { throw "$($linked.Package) build failed" }

        # these projects pin an explicit build.target in .cargo/config.toml
        # (to dodge the Windows command-line length limit), so output always
        # nests under the triple even for a same-arch host build.
        $exe = Join-Path $targetDir "x86_64-pc-windows-msvc\$profileDir\$($linked.Package).exe"
        if (Test-Path $exe) {
            Copy-Item -Force $exe (Join-Path $DestDir "$($linked.Package).exe")
        } else {
            Write-Host "  expected $($linked.Package) at $exe, not found - skipping copy" -ForegroundColor Yellow
        }
    }
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
        trunk build --release -v
        Pop-Location
        if ($LASTEXITCODE -ne 0) { throw "$module build failed" }
    }
}

if (-not $Dev) {
    # -Dev serves instead of building the site here (see below) so it can watch
    # for changes; every other mode just needs a one-shot build.
    Time-Step 'site' {
        trunk build --release -v
        if ($LASTEXITCODE -ne 0) { throw "site build failed" }
    }
}

if ($Deploy) {
    $deployDir = "deploy"
    Remove-Item -Recurse -Force $deployDir -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Path "$deployDir\site" -Force | Out-Null

    Time-Step 'burvy-dev/server' {
        Push-Location 'server'
        cargo build --release
        Pop-Location
        if ($LASTEXITCODE -ne 0) { throw "server build failed" }
    }
    Copy-Item -Force "server\target\x86_64-pc-windows-msvc\release\server.exe" "$deployDir\server.exe"

    Build-LinkedServers -DestDir $deployDir -Release -Time
}

$overall.Stop()

Write-Host "`nBuild times:"
foreach ($key in $timings.Keys) {
    Write-Host ("  {0,-20} {1,8:N1}s" -f $key, $timings[$key].TotalSeconds)
}
Write-Host ("  {0,-20} {1,8:N1}s" -f 'TOTAL', $overall.Elapsed.TotalSeconds)

if (-not $Dev) {
    Write-Host "`ndist/ is ready:"
    Get-ChildItem dist\*.wasm, dist\game\*.wasm, dist\life\*.wasm, dist\shooter\*.wasm, dist\floret\*.wasm, dist\venture\*.wasm -ErrorAction SilentlyContinue |
        ForEach-Object { "  {0,-22} {1,8:N1} MB" -f $_.Name, ($_.Length / 1MB) }
}

if ($Deploy) {
    Copy-Item -Recurse -Force "dist\*" "$deployDir\site\"

    Write-Host "`nDeploy folder ready at $deployDir\ (site\, server.exe, and every linked project's server.exe)"
    Write-Host "Note: server.exe expects certs at C:\burvy\certs\webtrans.burvy.dev\ on the target machine - not included here, on purpose."
    Write-Host "Each project's own go.ps1 -release still works standalone if you only want to redeploy one game."

    Invoke-Item $deployDir
}

if ($Dev) {
    Write-Host "`n==> starting local site server" -ForegroundColor Cyan
    Start-Process pwsh -ArgumentList @(
        '-NoExit', '-Command',
        "Set-Location '$PSScriptRoot'; trunk serve --release -v"
    )
    Start-Sleep -Seconds 2
    Start-Process 'http://localhost:8080'

    $devDir = Join-Path $PSScriptRoot 'dev-servers'
    Remove-Item -Recurse -Force $devDir -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Path $devDir | Out-Null

    Write-Host "`n==> building burvy-dev's own server (dev)" -ForegroundColor Cyan
    Push-Location 'server'
    cargo build
    $ok = ($LASTEXITCODE -eq 0)
    Pop-Location
    if (-not $ok) { throw "server build failed" }
    Copy-Item -Force 'server\target\x86_64-pc-windows-msvc\debug\server.exe' (Join-Path $devDir 'server.exe')

    Build-LinkedServers -DestDir $devDir

    Write-Host "`nSite live at http://localhost:8080 - dev servers gathered at $devDir\" -ForegroundColor Green
    Invoke-Item $devDir
}

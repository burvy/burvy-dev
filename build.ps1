# Builds the site, its wasm experiences, and the multiplayer servers.
#
#   .\build.ps1                     every wasm module, then the site, into dist\
#   .\build.ps1 amity-wasm          just amity's module (the others keep their
#                                   last build), then the site
#   .\build.ps1 -Modules @()        no modules, just the site (e.g. after a site-only
#                                   change: the modules already built are reused)
#
#   .\build.ps1 -Deploy             everything, gathered into deploy\ to copy to the
#                                   server machine as is:
#                                       deploy\burvy-dev\dist\   the site
#                                       deploy\server.exe        the chat server
#                                       deploy\amity-server.exe
#                                       deploy\floret-server.exe
#                                   Servers are RELEASE builds. Add -Modules to rebuild
#                                   only some modules (-Modules @() for none: the
#                                   modules already built go in)
#   .\build.ps1 -Deploy -Servers amity    only some servers (chat, amity, floret)
#
#   .\build.ps1 -Dev                serves the site on http://localhost:8080 and
#                                   gathers every server's DEV build (with its dev
#                                   flags, see $ServerList) into dev-servers\, to run
#                                   whichever one you're testing against it
#
# Certificates are never copied: server.exe expects C:\burvy\certs\webtrans.burvy.dev\,
# and each game server its own (see that project), on the machine they run on.
# Each linked project's own go.ps1 -release still works standalone, for redeploying
# one game without touching the rest.
param(
    [string[]] $Modules = @('life-wasm', 'amity-wasm', 'floret-wasm', 'venture-wasm'),
    [string[]] $Servers = @('chat', 'amity', 'floret'),
    [switch] $Deploy,
    [switch] $Dev
)

$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

# trunk only accepts true or false here, while other tools accept anything
if ($env:NO_COLOR) { $env:NO_COLOR = 'true' }

# Every server: burvy-dev's own chat server, and each linked project's game server.
# Dir is relative to this repo. DevArgs are the extra cargo flags its dev build
# needs (release builds just use --release). Add new servers here.
$ServerList = @(
    @{ Name = 'chat';   Dir = 'server';       Package = 'server';        DevArgs = @() },
    @{ Name = 'amity';  Dir = '..\amity';     Package = 'amity-server';  DevArgs = @('--features', 'dev-local') },
    @{ Name = 'floret'; Dir = '..\floret';    Package = 'floret-server'; DevArgs = @('--features', 'dev-local') }
)

$AllModules = @('life-wasm', 'amity-wasm', 'floret-wasm', 'venture-wasm')
# a full build rebuilds every module, unless told which
if (($Deploy -or $Dev) -and -not $PSBoundParameters.ContainsKey('Modules')) {
    $Modules = $AllModules
}

$timings = [ordered]@{}
function Time-Step {
    param([string] $Name, [scriptblock] $Body)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $Body
    $sw.Stop()
    $timings[$Name] = $sw.Elapsed
}

# Builds each server named in -Servers and copies its exe into $DestDir.
# -Release builds with --release; otherwise with the server's own DevArgs.
function Build-Servers {
    param([string] $DestDir, [switch] $Release)
    $profileDir = if ($Release) { 'release' } else { 'debug' }
    foreach ($server in $ServerList) {
        if ($Servers -notcontains $server.Name) { continue }

        $dir = Join-Path $PSScriptRoot $server.Dir
        if (-not (Test-Path $dir)) {
            Write-Host "  skipping $($server.Package): $dir not found" -ForegroundColor Yellow
            continue
        }

        # @(...) keeps a single flag an array: splatting a bare string would pass
        # it to cargo one character at a time
        $cargoArgs = @(if ($Release) { '--release' } else { $server.DevArgs })

        Write-Host "`n==> building $($server.Package) ($profileDir)" -ForegroundColor Cyan
        Time-Step $server.Package {
            Push-Location $dir
            try {
                cargo build -p $server.Package @cargoArgs
                if ($LASTEXITCODE -ne 0) { throw "$($server.Package) build failed" }
                $targetDir = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).target_directory
            } finally {
                Pop-Location
            }
            # every project pins build.target in .cargo/config.toml (dodging the
            # Windows command-line length limit), so output nests under the triple
            $exe = Join-Path $targetDir "x86_64-pc-windows-msvc\$profileDir\$($server.Package).exe"
            if (-not (Test-Path $exe)) { throw "expected $($server.Package) at $exe" }
            Copy-Item -Force $exe (Join-Path $DestDir "$($server.Package).exe")
        }
    }
}

$overall = [System.Diagnostics.Stopwatch]::StartNew()

# ---- wasm modules. Only the ones being rebuilt are cleared: the site copies
# whatever is in assets\, so the others keep their last build
foreach ($module in $Modules) {
    Time-Step $module {
        Remove-Item -Recurse -Force ('assets\' + ($module -replace '-wasm$', '')) -ErrorAction SilentlyContinue
        Push-Location "crates\$module"
        try {
            trunk build --release -v
            if ($LASTEXITCODE -ne 0) { throw "$module build failed" }
        } finally {
            Pop-Location
        }
    }
}

# ---- the site (-Dev serves it instead, below, so it can watch for changes)
if (-not $Dev) {
    Time-Step 'site' {
        trunk build --release -v
        if ($LASTEXITCODE -ne 0) { throw 'site build failed' }
    }
}

# ---- deploy\: the site under burvy-dev\dist\, every server exe beside it
if ($Deploy) {
    $deployDir = Join-Path $PSScriptRoot 'deploy'
    Remove-Item -Recurse -Force $deployDir -ErrorAction SilentlyContinue
    $siteDir = Join-Path $deployDir 'burvy-dev\dist'
    New-Item -ItemType Directory -Path $siteDir -Force | Out-Null
    Copy-Item -Recurse -Force 'dist\*' $siteDir

    Build-Servers -DestDir $deployDir -Release
}

$overall.Stop()

Write-Host "`nBuild times:"
foreach ($key in $timings.Keys) {
    Write-Host ('  {0,-20} {1,8:N1}s' -f $key, $timings[$key].TotalSeconds)
}
Write-Host ('  {0,-20} {1,8:N1}s' -f 'TOTAL', $overall.Elapsed.TotalSeconds)

if (-not $Dev) {
    Write-Host "`nWasm in dist\:"
    Get-ChildItem dist -Recurse -Filter *.wasm |
        ForEach-Object { '  {0,-24} {1,8:N1} MB' -f $_.Name, ($_.Length / 1MB) }
}

if ($Deploy) {
    Write-Host "`nDeploy folder ready at $deployDir\:" -ForegroundColor Green
    Get-ChildItem $deployDir |
        ForEach-Object { '  ' + $_.Name + $(if ($_.PSIsContainer) { '\' } else { '' }) }
    Write-Host 'Certificates are not included, on purpose: each server loads its own on the server machine.'
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
    Build-Servers -DestDir $devDir

    Write-Host "`nSite live at http://localhost:8080, dev servers gathered in $devDir\" -ForegroundColor Green
    Invoke-Item $devDir
}

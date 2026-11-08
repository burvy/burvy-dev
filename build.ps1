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
#                                       deploy\burvy-dev\dist\   the site (burvy.dev)
#                                       deploy\statesmc\dist\    the StatesMC site (statesmc.us)
#                                       deploy\server.exe        the chat server
#                                       deploy\amity-server.exe
#                                       deploy\sandbags-server.exe
#                                       deploy\floret-server.exe
#                                       deploy\plgroup-server.exe
#                                       deploy\rambl-server.exe   (Rambl: its web bundle is built first)
#                                   Servers are RELEASE builds. Add -Modules to rebuild
#                                   only some modules (-Modules @() for none: the
#                                   modules already built go in)
#   .\build.ps1 -Deploy -Servers amity    only some servers (chat, amity, sandbags, floret, plgroup, primer, rambl)
#   .\build.ps1 -Deploy -Servers rambl -RamblDir ..\ai-todo-release
#                                   Rambl from another checkout, relative to this
#                                   repo (default ..\ai-todo). Ship from a clean
#                                   checkout of origin/main: the build takes whatever
#                                   that folder holds, branch and edits included
#   .\build.ps1 -ServersOnly -Servers plgroup
#                                   just those servers' release exes into deploy\,
#                                   leaving everything else in deploy\ as it is
#                                   (menu.ps1 uses this)
#
#   .\build.ps1 -Dev               serves the site on http://localhost:8080 and
#                                   gathers every server's DEV build (with its dev
#                                   flags, see $ServerList) into dev-servers\, to run
#                                   whichever one you're testing against it
#
# Certificates are never copied: server.exe expects C:\burvy\certs\webtrans.burvy.dev\,
# and each game server its own (see that project), on the machine they run on.
# Each linked project's own go.ps1 -release still works standalone, for redeploying
# one game without touching the rest.
param(
    [string[]] $Modules = @('plgroup', 'life-wasm', 'amity-wasm', 'sandbags-wasm', 'floret-wasm', 'venture-wasm', 'primer-wasm'),
    [string[]] $Servers = @('chat', 'amity', 'sandbags', 'floret', 'plgroup', 'primer', 'rambl'),
    [string] $RamblDir = '..\ai-todo',
    [switch] $Deploy,
    [switch] $Dev,
    [switch] $ServersOnly
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
    @{ Name = 'sandbags'; Dir = '..\sandbags'; Package = 'sandbags-server'; DevArgs = @('--features', 'dev-local') },
    @{ Name = 'floret'; Dir = '..\floret';    Package = 'floret-server'; DevArgs = @('--features', 'dev-local') },
    @{ Name = 'plgroup'; Dir = 'plgroup-server'; Package = 'plgroup-server'; DevArgs = @('--features', 'dev-local') },
    @{ Name = 'primer'; Dir = '..\primer'; Package = 'primer-server'; DevArgs = @() },
    # Rambl's web bundle is embedded in its server, so the bundle is built first.
    @{ Name = 'rambl';  Dir = $RamblDir;      Package = 'rambl-server';  DevArgs = @(); Prebuild = {
        Push-Location (Join-Path $RamblDir 'web')
        try {
            trunk build --release
            if ($LASTEXITCODE -ne 0) { throw 'rambl web build failed' }
        } finally {
            Pop-Location
        }
    } }
)

$AllModules = @('plgroup', 'life-wasm', 'amity-wasm', 'sandbags-wasm', 'floret-wasm', 'venture-wasm', 'primer-wasm')
# a full build rebuilds every module, unless told which
if (($Deploy -or $Dev) -and -not $PSBoundParameters.ContainsKey('Modules')) {
    $Modules = $AllModules
}
if ($ServersOnly) { $Modules = @() }

$deployDir = Join-Path $PSScriptRoot 'deploy'

# `pwsh -File build.ps1 -Servers rambl,primer` passes 'rambl,primer' as one string,
# so split it here; a name that matches no server would otherwise build nothing and
# still report success
$Servers = @($Servers | ForEach-Object { $_ -split ',' } | ForEach-Object { $_.Trim() } | Where-Object { $_ })
$unknown = @($Servers | Where-Object { $ServerList.Name -notcontains $_ })
if ($unknown) { throw "unknown server(s): $($unknown -join ', '); expected $($ServerList.Name -join ', ')" }

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
            if ($server.Prebuild) { & $server.Prebuild }
            Push-Location $dir
            try {
                cargo build -p $server.Package @cargoArgs
                if ($LASTEXITCODE -ne 0) { throw "$($server.Package) build failed" }
                $targetDir = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).target_directory
            } finally {
                Pop-Location
            }
            # most projects pin build.target in .cargo/config.toml (dodging the
            # Windows command-line length limit), so output nests under the triple;
            # primer does not, so its exe is straight under the profile folder
            $exe = Join-Path $targetDir "x86_64-pc-windows-msvc\$profileDir\$($server.Package).exe"
            if (-not (Test-Path $exe)) { $exe = Join-Path $targetDir "$profileDir\$($server.Package).exe" }
            if (-not (Test-Path $exe)) { throw "expected $($server.Package) at $exe" }
            Copy-Item -Force $exe (Join-Path $DestDir "$($server.Package).exe")
        }
    }
}

$overall = [System.Diagnostics.Stopwatch]::StartNew()

# Sandbags (task 273): checks the site's sandbags module is the same build as the
# sandbags server going out with it, and publishes that build beside the module as
# sandbags\version.json. The game reads it after a failed connect (a stale tab after
# an update reloads itself once), and the site loads the module as
# sandbags-wasm.js?v=<module> from it, so no cache can serve an old one.
#   build:  the server's build id (`sandbags-server --build-id`); a client of another
#           build is refused, so the module must embed the same id (it shows it in its
#           could-not-connect text, which is how this finds it)
#   module: the first 16 hex of the wasm's sha256, so a client-only change, which keeps
#           the build id, still changes the module URL
function Publish-SandbagsVersion {
    param([string] $SiteDir, [string] $ServerExe)
    $moduleDir = Join-Path $SiteDir 'sandbags'
    $wasm = Join-Path $moduleDir 'sandbags-wasm_bg.wasm'
    if (-not (Test-Path $wasm)) { return }
    if (-not (Test-Path $ServerExe)) {
        Write-Host "  no sandbags version.json: $ServerExe was not built (-Servers without sandbags). Deploy the sandbags module only together with its server." -ForegroundColor Yellow
        return
    }
    $build = ((& $ServerExe --build-id) | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $build -notmatch '^[0-9a-f]{16}$') {
        throw "sandbags-server --build-id gave '$build' (exit $LASTEXITCODE), expected 16 hex digits"
    }
    # the id is ASCII text in the module's data; Latin-1 (code page 28591) maps every byte
    # to one char. GetEncoding and Get-FileHash work in Windows PowerShell 5.1 as well as 7
    $bytes = [System.IO.File]::ReadAllBytes($wasm)
    if (-not [System.Text.Encoding]::GetEncoding(28591).GetString($bytes).Contains($build)) {
        throw ("the sandbags module $wasm is not build $build, the build of $ServerExe. " +
            'Its players would be refused. Build sandbags-wasm and sandbags-server from the same sandbags checkout (both are in a plain -Deploy).')
    }
    $module = (Get-FileHash -LiteralPath $wasm -Algorithm SHA256).Hash.Substring(0, 16).ToLowerInvariant()
    $json = '{"build":"' + $build + '","module":"' + $module + '"}'
    [System.IO.File]::WriteAllText((Join-Path $moduleDir 'version.json'), $json + "`n")
    Write-Host "  sandbags: build $build, module $module -> sandbags\version.json" -ForegroundColor Green
    Write-Host '  Ship order: the sandbags server first (restart it), then the site files. Old tabs reload once the new version.json is up.'
}

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
if (-not $Dev -and -not $ServersOnly) {
    Time-Step 'site' {
        trunk build --release -v
        if ($LASTEXITCODE -ne 0) { throw 'site build failed' }
    }
}

# ---- deploy\: the site under burvy-dev\dist\, every server exe beside it
if ($Deploy) {
    Remove-Item -Recurse -Force $deployDir -ErrorAction SilentlyContinue
    $siteDir = Join-Path $deployDir 'burvy-dev\dist'
    New-Item -ItemType Directory -Path $siteDir -Force | Out-Null
    Copy-Item -Recurse -Force 'dist\*' $siteDir

    # statesmc.us: a separate site, built straight into its own deploy folder
    Time-Step 'statesmc' {
        Push-Location 'crates\statesmc'
        # its own target folder: trunk shares target\wasm-bindgen\ across the workspace,
        # and would otherwise pull burvy-dev's js snippets into statesmc's index.html
        $env:CARGO_TARGET_DIR = Join-Path $PSScriptRoot 'target\statesmc'
        try {
            trunk build --release -v --dist (Join-Path $deployDir 'statesmc\dist')
            if ($LASTEXITCODE -ne 0) { throw 'statesmc build failed' }
        } finally {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
            Pop-Location
        }
    }

    Build-Servers -DestDir $deployDir -Release
    Publish-SandbagsVersion -SiteDir $siteDir -ServerExe (Join-Path $deployDir 'sandbags-server.exe')
}

# ---- just some servers, added to whatever deploy\ already holds
if ($ServersOnly) {
    New-Item -ItemType Directory -Path $deployDir -Force | Out-Null
    Build-Servers -DestDir $deployDir -Release
    if ($Servers -contains 'sandbags') {
        Write-Host ("  sandbags: a new server build refuses the site's current module if the shared code changed, and that tab would only say 'not reachable'. " +
            'Ship it with a plain -Deploy so the module and version.json go out with it.') -ForegroundColor Yellow
    }
}

$overall.Stop()

Write-Host "`nBuild times:"
foreach ($key in $timings.Keys) {
    Write-Host ('  {0,-20} {1,8:N1}s' -f $key, $timings[$key].TotalSeconds)
}
Write-Host ('  {0,-20} {1,8:N1}s' -f 'TOTAL', $overall.Elapsed.TotalSeconds)

if (-not $Dev -and -not $ServersOnly) {
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

# Interactive build menu. Run: .\menu.ps1
#
# Lists everything that can be built and lets you pick which to rebuild this time,
# e.g. `1 7` (the plgroup embed, then burvy.dev) or `8 13` (statesmc.us and
# plgroup-server). Everything lands in deploy\, ready to copy to the server machine:
#     deploy\burvy-dev\dist\   burvy.dev
#     deploy\statesmc\dist\    statesmc.us (a separate site on its own domain)
#     deploy\*.exe             the servers
# Only what you pick is rebuilt; the rest of deploy\ is left as it was. Sites build
# straight into deploy\ instead of dist\, so this is safe while `trunk serve` runs.
# After a build it lists which files differ from what's live, i.e. what to copy.

$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

# trunk only accepts true or false here
if ($env:NO_COLOR) { $env:NO_COLOR = 'true' }

$SiteDist = 'deploy\burvy-dev\dist'
$StatesDist = 'deploy\statesmc\dist'

# Kind decides how it's built: a wasm module (crates\<Module>, copied into burvy.dev
# by its index.html), one of the two sites, StatesMC's adventure game, or a server
# (build.ps1's $ServerList name).
$Targets = @(
    @{ Label = 'plgroup embed (sites.psu.edu, and burvy.dev/plgroup)'; Kind = 'module'; Module = 'plgroup' }
    @{ Label = 'life (cellular automata)';   Kind = 'module'; Module = 'life-wasm' }
    @{ Label = 'amity';                      Kind = 'module'; Module = 'amity-wasm' }
    @{ Label = 'floret';                     Kind = 'module'; Module = 'floret-wasm' }
    @{ Label = 'venture';                    Kind = 'module'; Module = 'venture-wasm' }
    @{ Label = 'primer';                     Kind = 'module'; Module = 'primer-wasm' }
    @{ Label = 'burvy.dev site';             Kind = 'site' }
    @{ Label = 'statesmc.us site';           Kind = 'statesmc' }
    @{ Label = 'adventure (the game on statesmc.us, from ..\adventure)'; Kind = 'adventure' }
    @{ Label = 'chat server';                Kind = 'server'; Server = 'chat' }
    @{ Label = 'amity-server';               Kind = 'server'; Server = 'amity' }
    @{ Label = 'floret-server';              Kind = 'server'; Server = 'floret' }
    @{ Label = 'plgroup-server';             Kind = 'server'; Server = 'plgroup' }
    @{ Label = 'primer-server';              Kind = 'server'; Server = 'primer' }
    @{ Label = 'rambl-server (from ..\ai-todo, or another checkout)'; Kind = 'server'; Server = 'rambl' }
)

function Build-Module($module) {
    Write-Host "`n==> building $module" -ForegroundColor Cyan
    Push-Location "crates\$module"
    try {
        trunk build --release
        if ($LASTEXITCODE -ne 0) { throw "$module build failed" }
    } finally {
        Pop-Location
    }
}

function Build-Site {
    Write-Host "`n==> building burvy.dev into $SiteDist" -ForegroundColor Cyan
    trunk build --release --dist $SiteDist
    if ($LASTEXITCODE -ne 0) { throw 'burvy.dev build failed' }
}

function Build-StatesMC {
    Write-Host "`n==> building statesmc.us into $StatesDist" -ForegroundColor Cyan
    Push-Location 'crates\statesmc'
    # its own target folder: trunk shares target\wasm-bindgen\ across the workspace,
    # and would otherwise pull burvy-dev's js snippets into statesmc's index.html
    $env:CARGO_TARGET_DIR = Join-Path $PSScriptRoot 'target\statesmc'
    try {
        trunk build --release --dist (Join-Path $PSScriptRoot $StatesDist)
        if ($LASTEXITCODE -ne 0) { throw 'statesmc build failed' }
    } finally {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        Pop-Location
    }
}

# adventure is its own project; its build goes into statesmc's adventure-dist\,
# which the statesmc site copies (and which is committed in this repo)
function Build-Adventure {
    Write-Host "`n==> building adventure into crates\statesmc\adventure-dist" -ForegroundColor Cyan
    Push-Location '..\adventure'
    try {
        trunk build --release --dist (Join-Path $PSScriptRoot 'crates\statesmc\adventure-dist')
        if ($LASTEXITCODE -ne 0) { throw 'adventure build failed' }
    } finally {
        Pop-Location
    }
    Write-Host '  adventure-dist\ changed: commit it so statesmc keeps this build' -ForegroundColor Yellow
}

# Compares what index.html loads (plus $Extra files) against the live site by hash.
function Show-Changed($Dist, $Live, [string[]] $Extra = @()) {
    $index = Join-Path $Dist 'index.html'
    if (-not (Test-Path $index)) {
        Write-Host "nothing built yet in $Dist" -ForegroundColor Yellow
        return
    }

    $files = @('index.html')
    $files += [regex]::Matches((Get-Content $index -Raw), '(?:src|href)="/([^"]+)"') |
        ForEach-Object { $_.Groups[1].Value }
    $files += $Extra

    $tmp = New-TemporaryFile
    $changed = @()
    foreach ($file in $files | Sort-Object -Unique) {
        $local = Join-Path $Dist $file
        if (-not (Test-Path $local)) {
            Write-Host "  MISSING locally: $file (index.html points at it)" -ForegroundColor Red
            continue
        }
        try {
            # random query so neither Cloudflare nor anything else serves a stale copy
            Invoke-WebRequest "$Live/$file`?x=$(Get-Random)" -OutFile $tmp -UseBasicParsing
            if ($file -eq 'index.html') {
                # Cloudflare injects a bot-check link + script into <body> on the way out
                $strip = { param($s) $s -replace '(?s)<body>.*</body>', '<body></body>' -replace '\s', '' }
                $same = (& $strip (Get-Content $local -Raw)) -eq (& $strip (Get-Content $tmp -Raw))
            } else {
                $same = (Get-FileHash $local).Hash -eq (Get-FileHash $tmp).Hash
            }
        } catch {
            $same = $false
        }
        if (-not $same) { $changed += $file }
    }
    Remove-Item $tmp

    if ($changed.Count -eq 0) {
        Write-Host "`n$Live is up to date, nothing to copy." -ForegroundColor Green
    } else {
        Write-Host "`nCopy these from $Dist\ to the server ($Live):" -ForegroundColor Green
        $changed | ForEach-Object { "  $($_ -replace '/', '\')" }
    }
}

function Show-ChangedBurvy {
    $embed = @(Get-ChildItem (Join-Path $SiteDist 'plgroup-embed') -File -ErrorAction SilentlyContinue |
        Where-Object Name -ne '_module.html' |
        ForEach-Object { "plgroup-embed/$($_.Name)" })
    Show-Changed $SiteDist 'https://burvy.dev' $embed
}

function Show-ChangedStatesMC {
    if (-not (Test-Path $StatesDist)) {
        Write-Host "nothing built yet in $StatesDist" -ForegroundColor Yellow
        return
    }
    # a small site, so every file is checked (its css loads assets\ by relative paths)
    $root = (Resolve-Path $StatesDist).Path
    $all = @(Get-ChildItem $StatesDist -Recurse -File |
        ForEach-Object { $_.FullName.Substring($root.Length + 1) -replace '\\', '/' })
    Show-Changed $StatesDist 'https://statesmc.us' $all
}

# "1 3 7", "1,3", "10-15" or "a" for everything -> the picked targets' indexes
function Read-Picks($text) {
    if ($text -eq 'a') { return 0..($Targets.Count - 1) }
    $picks = foreach ($part in ($text -split '[\s,]+' | Where-Object { $_ })) {
        if ($part -match '^(\d+)-(\d+)$') { [int]$Matches[1]..[int]$Matches[2] }
        elseif ($part -match '^\d+$') { [int]$part }
        else { throw "not a number: '$part'" }
    }
    foreach ($n in $picks) {
        if ($n -lt 1 -or $n -gt $Targets.Count) { throw "no target $n" }
    }
    return @($picks | ForEach-Object { $_ - 1 } | Sort-Object -Unique)
}

function Build-Picked([int[]] $picked) {
    $chosen = @($picked | ForEach-Object { $Targets[$_] })
    $modules = @($chosen | Where-Object { $_.Kind -eq 'module' } | ForEach-Object { $_.Module })
    $servers = @($chosen | Where-Object { $_.Kind -eq 'server' } | ForEach-Object { $_.Server })
    $adventure = @($chosen | Where-Object { $_.Kind -eq 'adventure' }).Count -gt 0
    # burvy.dev copies the modules in, and statesmc copies adventure in, so those
    # sites are rebuilt along with them
    $site = @($chosen | Where-Object { $_.Kind -eq 'site' }).Count -gt 0 -or $modules.Count -gt 0
    $states = @($chosen | Where-Object { $_.Kind -eq 'statesmc' }).Count -gt 0 -or $adventure

    $plan = @($modules) + @(if ($site) { 'burvy.dev' }) + @(if ($adventure) { 'adventure' }) +
        @(if ($states) { 'statesmc.us' }) + @($servers | ForEach-Object { "$_ server" })
    Write-Host "`nbuilding: $($plan -join ', ')" -ForegroundColor Cyan

    $ramblDir = '..\ai-todo'
    if ($servers -contains 'rambl') {
        $answer = Read-Host "Rambl checkout to build from, relative to burvy-dev (enter for $ramblDir)"
        if ($answer) { $ramblDir = $answer }
    }

    foreach ($module in $modules) { Build-Module $module }
    if ($site) { Build-Site }
    if ($adventure) { Build-Adventure }
    if ($states) { Build-StatesMC }
    if ($servers.Count -gt 0) {
        & .\build.ps1 -ServersOnly -Servers $servers -RamblDir $ramblDir
    }

    if ($site) { Show-ChangedBurvy }
    if ($states) { Show-ChangedStatesMC }
}

while ($true) {
    Write-Host ''
    Write-Host 'burvy-dev: pick what to rebuild' -ForegroundColor Cyan
    for ($i = 0; $i -lt $Targets.Count; $i++) {
        if ($i -eq 0) { Write-Host '  wasm modules (burvy.dev is rebuilt with them)' -ForegroundColor DarkGray }
        if ($Targets[$i].Kind -eq 'site') { Write-Host '  sites' -ForegroundColor DarkGray }
        if ($Targets[$i].Server -eq 'chat') { Write-Host '  servers (release exes into deploy\)' -ForegroundColor DarkGray }
        Write-Host ('  {0,2}  {1}' -f ($i + 1), $Targets[$i].Label)
    }
    Write-Host '  e.g. "1 7" or "10-15"; a = everything'
    Write-Host '   l  list what to copy (no build)'
    Write-Host '   o  open the deploy folder'
    Write-Host '   q  quit'
    $choice = (Read-Host 'choose').Trim()

    try {
        switch ($choice) {
            'q' { return }
            'l' { Show-ChangedBurvy; Show-ChangedStatesMC }
            'o' { New-Item -ItemType Directory 'deploy' -Force | Out-Null; Invoke-Item 'deploy' }
            '' { }
            default { Build-Picked (Read-Picks $choice) }
        }
    } catch {
        Write-Host $_ -ForegroundColor Red
    }
}

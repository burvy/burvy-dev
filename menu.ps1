# Interactive menu for the common build/deploy jobs. Run: .\menu.ps1
#
# Everything here builds the site straight into deploy\burvy-dev\dist\ instead of
# dist\, so it's safe to use while `trunk serve` is running (serve owns dist\).
# After a build it lists which files differ from what's live on burvy.dev, i.e.
# exactly what to copy to the server machine.

$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

# trunk only accepts true or false here
if ($env:NO_COLOR) { $env:NO_COLOR = 'true' }

$SiteDist = 'deploy\burvy-dev\dist'
$Live = 'https://burvy.dev'

function Build-Embed {
    Write-Host "`n==> building the plgroup embed (sites.psu.edu)" -ForegroundColor Cyan
    Push-Location 'crates\plgroup'
    try {
        trunk build --release
        if ($LASTEXITCODE -ne 0) { throw 'plgroup embed build failed' }
    } finally {
        Pop-Location
    }
}

function Build-Site {
    Write-Host "`n==> building the site into $SiteDist" -ForegroundColor Cyan
    trunk build --release --dist $SiteDist
    if ($LASTEXITCODE -ne 0) { throw 'site build failed' }
}

# Every file index.html loads, plus the embed, compared against burvy.dev by hash.
function Show-Changed {
    $index = Join-Path $SiteDist 'index.html'
    if (-not (Test-Path $index)) {
        Write-Host "nothing built yet in $SiteDist" -ForegroundColor Yellow
        return
    }

    $files = @('index.html')
    $files += [regex]::Matches((Get-Content $index -Raw), '(?:src|href)="/([^"]+)"') |
        ForEach-Object { $_.Groups[1].Value }
    $files += Get-ChildItem (Join-Path $SiteDist 'plgroup-embed') -File |
        Where-Object Name -ne '_module.html' |
        ForEach-Object { "plgroup-embed/$($_.Name)" }

    $tmp = New-TemporaryFile
    $changed = @()
    foreach ($file in $files | Sort-Object -Unique) {
        $local = Join-Path $SiteDist $file
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
        Write-Host "`nburvy.dev is up to date, nothing to copy." -ForegroundColor Green
    } else {
        Write-Host "`nCopy these from $SiteDist\ to the server:" -ForegroundColor Green
        $changed | ForEach-Object { "  $($_ -replace '/', '\')" }
    }
}

while ($true) {
    Write-Host ''
    Write-Host 'burvy-dev' -ForegroundColor Cyan
    Write-Host '  1  plgroup: build embed + site, then list what to copy'
    Write-Host '  2  site only (no embed), then list what to copy'
    Write-Host '  3  list what to copy (no build)'
    Write-Host '  4  open the deploy folder'
    Write-Host '  5  full deploy: every module + servers (build.ps1 -Deploy; stop trunk serve first)'
    Write-Host '  q  quit'
    $choice = Read-Host 'choose'

    try {
        switch ($choice) {
            '1' { Build-Embed; Build-Site; Show-Changed }
            '2' { Build-Site; Show-Changed }
            '3' { Show-Changed }
            '4' { Invoke-Item $SiteDist }
            '5' { & .\build.ps1 -Deploy }
            'q' { return }
            default { Write-Host "no option '$choice'" -ForegroundColor Yellow }
        }
    } catch {
        Write-Host $_ -ForegroundColor Red
    }
}

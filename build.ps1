# Full release build. Each module's Trunk.toml already points its output at
# assets/, so these are plain trunk builds — the only thing that matters is
# the order: copy-dir needs assets/{game,life} populated before the site
# build runs.
#
# Day to day you don't need this. Use `trunk serve` for site work, and only
# rebuild a module when you've changed web-fps or life-v2.
Set-Location $PSScriptRoot

# trunk's --no-color flag insists on true/false, so the conventional
# NO_COLOR=1 that some terminals set makes it exit before building
if ($env:NO_COLOR) { $env:NO_COLOR = 'true' }

# wipe first — trunk doesn't clean these, so a renamed or removed output from
# an older build would otherwise linger and keep getting copied into dist
#
# (this does NOT remove the stray snippets/ dir in each module — trunk stages
# all three crates through the shared target/wasm-bindgen/, so burvy-dev's
# loader snippet gets copied along with each module. It's ~350 bytes and
# nothing references it; separating the staging dirs would force full
# recompiles, which is a far worse trade.)
Remove-Item -Recurse -Force assets\game, assets\life -ErrorAction SilentlyContinue

foreach ($module in 'life-wasm', 'game-wasm') {
    Push-Location "crates\$module"
    trunk build --release
    Pop-Location
    # without this the site would still build and quietly ship a stale module
    if ($LASTEXITCODE -ne 0) { throw "$module build failed" }
}

trunk build --release
if ($LASTEXITCODE -ne 0) { throw "site build failed" }

Write-Host "`ndist/ is ready to deploy:"
Get-ChildItem dist\*.wasm, dist\game\*.wasm, dist\life\*.wasm |
    ForEach-Object { "  {0,-22} {1,8:N1} MB" -f $_.Name, ($_.Length / 1MB) }

param([switch]$SansTests)
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    if (-not $SansTests) {
        & cargo test --workspace --locked
        if ($LASTEXITCODE -ne 0) { throw 'Les tests Rust ont echoue.' }
    }
    & cargo build --locked --package leviathan-client --bin LeviathanRust
    if ($LASTEXITCODE -ne 0) { throw 'Compilation impossible.' }
    $dist = Join-Path $PSScriptRoot 'dist'
    New-Item -ItemType Directory -Force -Path $dist | Out-Null
    Copy-Item (Join-Path $PSScriptRoot 'target\debug\LeviathanRust.exe') (Join-Path $dist 'LeviathanRust-0.3.exe') -Force
    Copy-Item (Join-Path $PSScriptRoot 'assets') $dist -Recurse -Force
    if (-not (Test-Path (Join-Path $dist 'settings.toml'))) {
        Copy-Item (Join-Path $PSScriptRoot 'settings.toml') (Join-Path $dist 'settings.toml')
    }
    Write-Output "Pret : $dist\LeviathanRust-0.3.exe"
    Write-Output 'Les reglages existants dans dist ne sont pas remplaces.'
} finally { Pop-Location }

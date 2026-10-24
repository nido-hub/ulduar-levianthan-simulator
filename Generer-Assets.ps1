# Rebuild the original procedural vehicle into the self-contained GLB used by the client.
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    & cargo build --locked --package leviathan-client --bin LeviathanRust
    if ($LASTEXITCODE -ne 0) { throw 'Compilation impossible.' }
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = Join-Path $PSScriptRoot 'target\debug\LeviathanRust.exe'
    $info.Arguments = '--bake-assets'
    $info.WorkingDirectory = $PSScriptRoot
    $info.UseShellExecute = $false
    $process = [System.Diagnostics.Process]::Start($info)
    if (-not $process.WaitForExit(60000)) { $process.Kill(); throw 'Generation trop longue.' }
    if ($process.ExitCode -ne 0) { throw 'Echec de generation. Voir target\debug\art-bake.txt et panic.log.' }
    Get-Content (Join-Path $PSScriptRoot 'target\debug\art-bake.txt')
    $process.Dispose()
    Write-Output 'GLB regenere dans assets\models. Executer Construire.ps1 pour le copier dans dist.'
} finally { Pop-Location }

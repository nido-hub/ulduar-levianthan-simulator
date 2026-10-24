param([switch]$Menu, [switch]$Art, [switch]$Fallback, [switch]$Combat, [ValidateSet('demo','demo-passenger','siege','gunner','moto')][string]$Role='demo', [ValidateRange(20,120)][int]$Seconds=20)
$ErrorActionPreference = 'Stop'
$dist = Join-Path $PSScriptRoot 'dist'
$exe = Join-Path $dist 'LeviathanRust-0.3.exe'
if (-not (Test-Path $exe)) { throw 'Compiler avec Construire.ps1 avant le test natif.' }
$started = [DateTime]::UtcNow
$info = New-Object System.Diagnostics.ProcessStartInfo
$info.FileName = $exe
$info.WorkingDirectory = $dist
$info.Arguments = if ($Menu) { '--smoke-menu' } elseif ($Art) { '--art-preview' } elseif ($Fallback) { '--smoke-fallback' } elseif ($Combat) { "--smoke-combat --role=$Role --combat-seconds=$Seconds" } else { '--smoke-test' }
$info.UseShellExecute = $false
$info.RedirectStandardOutput = $true
$info.RedirectStandardError = $true
$process = New-Object System.Diagnostics.Process
$process.StartInfo = $info
$null = $process.Start()
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
if (-not $process.WaitForExit([Math]::Max(90000,($Seconds+60)*1000))) {
    $process.Kill()
    throw 'Le test natif a depasse son delai ; seul son processus a ete arrete.'
}
$process.WaitForExit()
[System.IO.File]::WriteAllText((Join-Path $dist 'native-stdout.log'), $stdout.GetAwaiter().GetResult())
[System.IO.File]::WriteAllText((Join-Path $dist 'native-stderr.log'), $stderr.GetAwaiter().GetResult())
if ($process.ExitCode -ne 0) { throw "Le test natif a echoue (code $($process.ExitCode)). Voir dist\native-stderr.log et dist\panic.log." }
$stem = if ($Menu) { 'validation-menu' } elseif ($Art) { 'validation-art' } elseif ($Fallback) { 'validation-fallback' } elseif ($Combat) { 'validation-combat' } else { 'validation' }
$report = Get-Item (Join-Path $dist "$stem.txt")
$capture = Get-Item (Join-Path $dist "$stem.png")
if ($report.LastWriteTimeUtc -lt $started -or $capture.LastWriteTimeUtc -lt $started) { throw 'Les fichiers de validation ne sont pas nouveaux.' }
$text = Get-Content $report.FullName -Raw
if ($text -notmatch 'native_smoke=PASS') { throw "Echec : $text" }
Write-Output $text
Write-Output "Capture GPU native : $($capture.FullName)"
$process.Dispose()

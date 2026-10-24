param(
    [string]$Client = (Split-Path $PSScriptRoot -Parent),
    [Parameter(Mandatory=$true)][string]$StormLib
)
$ErrorActionPreference = 'Stop'
# Build-time only: Python 3 + Pillow, 64-bit Unicode StormLib DLL.
# Official StormLib releases: https://github.com/ladislav-zezula/StormLib/releases
& python (Join-Path $PSScriptRoot 'tools\import_wow.py') --client $Client --stormlib $StormLib
if ($LASTEXITCODE -ne 0) { throw 'Import WoW echoue ; le client est reste en lecture seule.' }
Write-Output 'Ressources copiees. Executer Construire.ps1 pour mettre a jour la livraison autonome.'

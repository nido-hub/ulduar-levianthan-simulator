@echo off
setlocal
if not exist "%~dp0dist\LeviathanRust-0.3.exe" (
  echo L'executable n'est pas encore construit.
  echo Lancez Construire.ps1 avec PowerShell pour le compiler.
  pause
  exit /b 1
)
start "" /D "%~dp0dist" "%~dp0dist\LeviathanRust-0.3.exe"

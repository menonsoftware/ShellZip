# Register ShellZip as an Open With handler (and optionally the default).
# Prefer: shellzip --integrate
# Usage: .\scripts\register-open-with.ps1 [-Exe path\to\shellzip.exe] [-Default]

param(
    [string]$Exe = "",
    [switch]$Default
)

$ErrorActionPreference = "Stop"

if (-not $Exe) {
    $cmd = Get-Command shellzip -ErrorAction SilentlyContinue
    if ($cmd) {
        $Exe = $cmd.Source
    } else {
        throw "Pass -Exe with the path to shellzip.exe"
    }
}
$Exe = (Resolve-Path $Exe).Path

& $Exe --integrate
if ($LASTEXITCODE -ne 0) {
    throw "shellzip --integrate failed with exit $LASTEXITCODE"
}

if ($Default) {
    Write-Output "ProgIDs registered. If Windows keeps another default, open Settings → Default apps and choose ShellZip for each archive type."
} else {
    Write-Output "Registered ShellZip associations via --integrate ($Exe)"
}

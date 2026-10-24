# Register the ShellZip.Archive ProgID and its open command.
# Pass -Default to make ShellZip the user-level handler for common archive extensions.
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
$command = "`"$Exe`" `"%1`""

New-Item -Path "HKCU:\Software\Classes\ShellZip.Archive" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\ShellZip.Archive" -Name "(default)" -Value "ShellZip Archive"
New-Item -Path "HKCU:\Software\Classes\ShellZip.Archive\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\ShellZip.Archive\shell\open\command" -Name "(default)" -Value $command

$extensions = @(".zip", ".rar", ".7z", ".tar", ".gz", ".zst", ".tgz", ".tzst", ".sfx")
foreach ($ext in $extensions) {
    $with = "HKCU:\Software\Classes\$ext\OpenWithProgids"
    New-Item -Path $with -Force | Out-Null
    New-ItemProperty -Path $with -Name "ShellZip.Archive" -PropertyType String -Value "" -Force | Out-Null
    if ($Default) {
        Set-ItemProperty -Path "HKCU:\Software\Classes\$ext" -Name "(default)" -Value "ShellZip.Archive"
    }
}

Write-Output "Registered ShellZip.Archive -> $command"

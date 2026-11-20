param(
    [ValidateSet('fast', 'release', 'debug')][string]$Profile = 'fast',
    [switch]$NoBuild,
    [Parameter(ValueFromRemainingArguments = $true)][string[]]$AppArgs
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = if ($Profile -eq 'debug') {
    Join-Path $root 'target/debug/rustyfiles.exe'
} else {
    Join-Path $root "target/$Profile/rustyfiles.exe"
}

# Stop before building: Windows holds a lock on the running exe, so a rebuild cannot overwrite it.
$running = @(Get-Process -Name rustyfiles -ErrorAction SilentlyContinue)
if ($running.Count -gt 0) {
    $ids = $running.Id
    $running | Stop-Process -Force
    Wait-Process -Id $ids -Timeout 10 -ErrorAction SilentlyContinue
}

if (-not $NoBuild) {
    $buildArgs = if ($Profile -eq 'debug') { @('build', '-p', 'rf-app') } else { @('build', '--profile', $Profile, '-p', 'rf-app') }
    Push-Location $root
    try {
        & cargo @buildArgs
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
    } finally {
        Pop-Location
    }
}

if (-not (Test-Path $exe)) {
    throw "No executable at $exe. Run without -NoBuild first."
}

$startArgs = @{ FilePath = $exe; WorkingDirectory = $root }
if ($AppArgs) { $startArgs['ArgumentList'] = $AppArgs }
Start-Process @startArgs
Write-Host "Started $exe"

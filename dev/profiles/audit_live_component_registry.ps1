[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$TypeCatalog,
    [string]$Profile = (Join-Path $PSScriptRoot '..\..\kfc-runtime\profiles\enshrouded\client\1076226.json'),
    [uint32]$ProcessId = 0,
    [string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$buildRoot = Join-Path $repoRoot 'build/native-runtime'
$buildTool = Join-Path $buildRoot 'tools\kfc-runtime-audit-live-component-registry.exe'
$packageTool = Join-Path $PSScriptRoot 'kfc-runtime-audit-live-component-registry.exe'
$TypeCatalog = [IO.Path]::GetFullPath($TypeCatalog)
$Profile = [IO.Path]::GetFullPath($Profile)

if (-not (Test-Path -LiteralPath $TypeCatalog)) { throw "Type catalog not found: $TypeCatalog" }
if (-not (Test-Path -LiteralPath $Profile)) { throw "Build profile not found: $Profile" }

if ($ProcessId -eq 0) {
    $game = Get-Process -Name enshrouded -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and [IO.Path]::GetFullPath($_.Path) -match '\\enshrouded\.exe$' } |
        Select-Object -First 1
    if (-not $game) {
        throw 'Enshrouded is not running. Start it, load into a world, then run this script again.'
    }
    $ProcessId = [uint32]$game.Id
}

if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
    throw "No running process has PID $ProcessId."
}

if (Test-Path -LiteralPath $buildTool) {
    $tool = $buildTool
} elseif (Test-Path -LiteralPath $packageTool) {
    $tool = $packageTool
} else {
    throw 'Build the audit explicitly first: cmake --build build/native-runtime --config Release --target kfc-runtime-audit-live-component-registry'
}

if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path $PSScriptRoot 'captures\component-registry-audit'
}
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$log = Join-Path $OutputDirectory "registry-audit-$ProcessId-$stamp.log"
$mappingFile = Join-Path $OutputDirectory "registry-candidates-$ProcessId-$stamp.tsv"

Write-Host "Scanning Enshrouded PID $ProcessId; this reads process memory and does not modify the game."
Write-Host "Full output: $log"
& $tool $ProcessId $TypeCatalog $Profile 2>&1 | Tee-Object -FilePath $log
if ($LASTEXITCODE -ne 0) { throw "Registry audit exited with code $LASTEXITCODE. See $log" }

$mappings = Get-Content -LiteralPath $log | Where-Object { $_ -match '^new_mapping\t' }
if ($mappings) {
    Set-Content -LiteralPath $mappingFile -Value $mappings -Encoding utf8
    Write-Host "Candidate mappings: $mappingFile"
} else {
    Write-Warning 'No additional table candidate passed the known-profile cross-check. Keep the full log for the next reverse-engineering pass.'
}

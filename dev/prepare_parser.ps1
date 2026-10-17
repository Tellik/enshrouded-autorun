[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$parserRoot = Join-Path $repositoryRoot 'kfc-parser'
$patchPath = Join-Path $PSScriptRoot 'kfc-parser-integration.patch'
if (-not (Test-Path -LiteralPath (Join-Path $parserRoot '.git'))) {
    throw 'Initialize the kfc-parser Git submodule before preparing its local integration.'
}
$revision = & git -C $parserRoot rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $revision -ne '34a156f6614f78c3340c07e4750afa400349abeb') {
    throw 'The parser integration patch requires the pinned 34a156f6 submodule revision.'
}
& git -C $parserRoot apply --reverse --check $patchPath 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Output 'KFC parser integration is already prepared.'
    exit 0
}
& git -C $parserRoot apply --check $patchPath
if ($LASTEXITCODE -ne 0) {
    throw 'The parser integration patch does not apply cleanly. Review local submodule changes before building.'
}
& git -C $parserRoot apply $patchPath
if ($LASTEXITCODE -ne 0) { throw 'Failed to prepare the local KFC parser integration.' }
Write-Output 'KFC parser integration prepared from the recorded patch.'

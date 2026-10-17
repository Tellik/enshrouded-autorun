[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$RegistryCapture
)
$ErrorActionPreference = 'Stop'
$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$RegistryCapture = (Resolve-Path -LiteralPath $RegistryCapture).Path
$previousCorpus = $env:SHROUDFORGE_TEST_FUNCTIONS
Push-Location -LiteralPath $repositoryRoot
try {
    $env:SHROUDFORGE_TEST_FUNCTIONS = $RegistryCapture
    & cargo test -p shroudforge-api --lib --offline production_callback_binding_corpus -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'Lua callback corpus failed' }
} finally {
    $env:SHROUDFORGE_TEST_FUNCTIONS = $previousCorpus
    Pop-Location
}

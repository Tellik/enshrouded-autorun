$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
Push-Location $root
try {
    & npm run site:check
    if ($LASTEXITCODE -ne 0) { throw 'ShroudForge website build or validation failed.' }
} finally {
    Pop-Location
}

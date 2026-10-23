$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$output = Join-Path $root 'build/x64'
New-Item -ItemType Directory -Force -Path $output | Out-Null
$bootstrapTests = Join-Path $root 'modules/entrypoints/windows-bootstrap/tests/run.ps1'
& pwsh -NoProfile -File $bootstrapTests
if ($LASTEXITCODE -ne 0) { throw 'ShroudForge Windows bootstrap tests failed.' }

& cargo test --manifest-path (Join-Path $root 'Cargo.toml') --release --workspace
if ($LASTEXITCODE -ne 0) { throw 'ShroudForge workspace tests failed.' }

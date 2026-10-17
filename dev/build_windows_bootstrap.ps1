$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$output = Join-Path $root 'build/x64'
New-Item -ItemType Directory -Force -Path $output | Out-Null
$visualStudio = & (Join-Path $PSScriptRoot 'find_visual_studio.ps1')
$msbuild = Join-Path $visualStudio 'MSBuild/Current/Bin/MSBuild.exe'
$bootstrap = Join-Path $root 'entrypoints\windows-bootstrap\ShroudForge.Bootstrap.vcxproj'
& $msbuild $bootstrap /m /t:Build /p:Configuration=Release /p:Platform=x64 /p:OutDir="$output\"
if ($LASTEXITCODE -ne 0) { throw 'ShroudForge Windows bootstrap build failed.' }

$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$output = Join-Path $root 'build/x64'
New-Item -ItemType Directory -Force -Path $output | Out-Null
$visualStudio = & (Join-Path $PSScriptRoot 'find_visual_studio.ps1')
$msbuild = Join-Path $visualStudio 'MSBuild\Current\Bin\MSBuild.exe'
$runtimeCmake = Join-Path $visualStudio 'Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin/cmake.exe'
& $runtimeCmake -S $root -B (Join-Path $root 'build/native-runtime') -A x64
if ($LASTEXITCODE -ne 0) { throw 'KFC Runtime configuration failed.' }
& $runtimeCmake --build (Join-Path $root 'build/native-runtime') --config Release
if ($LASTEXITCODE -ne 0) { throw 'KFC Runtime build failed.' }

Copy-Item -LiteralPath (Join-Path $root 'build/native-runtime/bin/kfc-runtime.dll') -Destination $output -Force

param(
    [string]$BuildNumber = $(if ($env:SHROUDFORGE_BUILD_NUMBER) { $env:SHROUDFORGE_BUILD_NUMBER } else { 'dev' }),
    [switch]$SkipTests,
    [switch]$UseInstalledDependencies,
    [switch]$SkipUiBuild,
    [string]$CargoTargetDir = 'target'
)

$ErrorActionPreference = 'Stop'
$actions = Join-Path $PSScriptRoot 'dev'
$cargoTargetPath = if ([IO.Path]::IsPathRooted($CargoTargetDir)) { [IO.Path]::GetFullPath($CargoTargetDir) } else { [IO.Path]::GetFullPath((Join-Path $PSScriptRoot $CargoTargetDir)) }
if ($BuildNumber -notmatch '^[A-Za-z0-9._-]+$') { throw 'BuildNumber contains invalid path characters.' }
& (Join-Path $actions 'prepare_parser.ps1')
$version = (Get-Content -LiteralPath (Join-Path $PSScriptRoot 'VERSION') -Raw).Trim()
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw "VERSION must use MAJOR.MINOR.PATCH: $version" }
& (Join-Path $actions 'build_modloader_ui.ps1') -UseInstalledDependencies:$UseInstalledDependencies -SkipUiBuild:$SkipUiBuild
& (Join-Path $actions 'build_runtime.ps1')
& (Join-Path $actions 'build_windows_proxies.ps1') -CargoTargetDir $cargoTargetPath
if (-not $SkipTests) {
    & (Join-Path $actions 'run_workspace_tests.ps1')
}
& (Join-Path $actions 'build_modloader.ps1') -CargoTargetDir $cargoTargetPath -UseInstalledDependencies -SkipUiBuild
& (Join-Path $actions 'build_windows_bootstrap.ps1')
& (Join-Path $actions 'package_release.ps1') -BuildNumber $BuildNumber -Version $version

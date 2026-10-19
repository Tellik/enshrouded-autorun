param(
    [string]$Python = 'python',
    [string]$ClientDirectory = 'C:\Program Files (x86)\Steam\steamapps\common\Enshrouded',
    [string]$ServerDirectory = 'C:\Program Files (x86)\Steam\steamapps\common\EnshroudedServer',
    [ValidateSet('client', 'server', 'both')][string]$Target = 'both',
    [string]$OutputDirectory,
    [switch]$Offline,
    [string]$NativeVerifier,
    [string]$ResourceExporter,
    [switch]$VerifyBufferAdapters,
    [Alias('InspectModOrigins')][switch]$VerifyRuntimeOrigins,
    [switch]$RunCallbackCorpusTests
)
$ErrorActionPreference = 'Stop'
$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$runtimeRoot = Join-Path $repositoryRoot 'modules/game-runtime'
$pythonRoot = Join-Path $repositoryRoot 'modules/runtime-diagnostics/diagnostics-jobs'
if (-not $NativeVerifier) {
    $NativeVerifier = Join-Path $repositoryRoot 'build/native-runtime/tools/kfc-runtime-verify-component-registry.exe'
}
if (-not $Offline -and -not (Test-Path -LiteralPath $NativeVerifier)) {
    throw 'Build the shared production verifier first: cmake --build build/native-runtime --config Release --target kfc-runtime-verify-component-registry'
}
if (-not $OutputDirectory) {
    $OutputDirectory = Join-Path $repositoryRoot ('dev/profiles/captures/discovery-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
}
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if ($VerifyRuntimeOrigins) {
    if (-not $ResourceExporter) {
        $ResourceExporter = Join-Path $repositoryRoot 'target/debug/examples/export_kfc_resources.exe'
    }
    if (-not (Test-Path -LiteralPath $ResourceExporter)) {
        throw 'Build the exporter explicitly first: cargo build -p shroudforge-api --example export_kfc_resources --offline, or supply -ResourceExporter.'
    }
}
foreach ($kind in @('client', 'server')) {
    if ($Target -ne 'both' -and $Target -ne $kind) { continue }
    $directory = if ($kind -eq 'client') { $ClientDirectory } else { $ServerDirectory }
    $stem = if ($kind -eq 'client') { 'enshrouded' } else { 'enshrouded_server' }
    $exe = (Resolve-Path -LiteralPath (Join-Path $directory "$stem.exe")).Path
    $arguments = @($exe, '--functions', '--out', (Join-Path $OutputDirectory $kind))
    # A previous profile supplies review anchors only. The scanner checks its PE
    # identity independently and discovers reflection/allocated tables without it.
    $profiles = @(Get-ChildItem -LiteralPath (Join-Path $runtimeRoot "profiles/enshrouded/$kind") -Filter '*.json')
    if ($profiles.Count -eq 1) { $arguments += @('--profile', $profiles[0].FullName) }
    if (-not $Offline) {
        $processes = @(Get-Process -Name $stem -ErrorAction SilentlyContinue |
            Where-Object { $_.Path -and [IO.Path]::GetFullPath($_.Path) -eq $exe })
        if ($processes.Count -ne 1) {
            throw "Expected one running $exe; found $($processes.Count). Use -Offline for EXE analysis only."
        }
        $arguments += @('--pid', [string]$processes[0].Id)
    }
    Push-Location -LiteralPath $pythonRoot
    try {
        & $Python -m game_inspection.capture.discover_runtime_build @arguments
    } finally {
        Pop-Location
    }
    if ($LASTEXITCODE -ne 0) { throw "$kind discovery failed with exit code $LASTEXITCODE" }
    $capture = [IO.Path]::GetFullPath((Join-Path $OutputDirectory $kind))
    $sha256 = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
    $matchingProfiles = @($profiles | Where-Object {
        (Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json).image.sha256 -eq $sha256
    })
    if ($VerifyRuntimeOrigins) {
        & $ResourceExporter $exe (Join-Path $capture 'resources.json')
        if ($LASTEXITCODE -ne 0) { throw "$kind KFC resource inspection failed" }
        if ($matchingProfiles.Count -eq 1) {
            Push-Location -LiteralPath $pythonRoot
            try {
                & $Python -m game_inspection.verify.verify_runtime_origins $exe --profile $matchingProfiles[0].FullName `
                    --functions (Join-Path $capture 'functions.json') --resources (Join-Path $capture 'resources.json') `
                    --out (Join-Path $capture 'mod-origins.json')
            } finally {
                Pop-Location
            }
            if ($LASTEXITCODE -ne 0) { throw "$kind mod origin audit failed" }
        } else { Write-Warning "$kind has no unique exact profile; original resources and function descriptors are captured, mod interventions remain unverified" }
    }
    if (-not $Offline) {
        $capture = [IO.Path]::GetFullPath((Join-Path $OutputDirectory $kind))
        $components = Get-Content -LiteralPath (Join-Path $capture 'components.json') -Raw | ConvertFrom-Json
        $managers = @($components.registries | ForEach-Object { $_.managers } | Sort-Object distinctLayouts -Descending)
        if ($managers.Count -eq 0) { throw "$kind has no validated live registry manager; the partial capture is retained at $capture" }
        $verification = Join-Path $capture 'provider-verification.json'
        $verifierArguments = @([string]$processes[0].Id, ($managers[0].address -replace '^0x', ''), $verification)
        if ($matchingProfiles.Count -eq 1) { $verifierArguments += $matchingProfiles[0].FullName }
        & $NativeVerifier @verifierArguments
        if ($LASTEXITCODE -ne 0) { throw "$kind production registry verification failed" }
        if ($RunCallbackCorpusTests) {
            & (Join-Path $PSScriptRoot '..\checks\run_callback_corpus_tests.ps1') -RegistryCapture $verification
        }
        if ($VerifyBufferAdapters) {
            Push-Location -LiteralPath $pythonRoot
            try {
                & $Python -m game_inspection.checks.verify_component_copy_adapters $exe `
                    --registry $verification --adapters "$verification.adapters.json" `
                    --out (Join-Path $capture 'adapter-verification.json')
            } finally {
                Pop-Location
            }
            if ($LASTEXITCODE -ne 0) { throw "$kind isolated callback differential verification failed" }
        }
        Push-Location -LiteralPath $pythonRoot
        try {
            & $Python -m game_inspection.capture.export_runtime_bindings $exe `
                --capture $capture --out (Join-Path $capture 'runtime-bindings.json') `
                --index (Join-Path $capture 'runtime-index.json')
        } finally {
            Pop-Location
        }
        if ($LASTEXITCODE -ne 0) { throw "$kind unified runtime catalog export failed" }
    }
}
if ($Target -eq 'both') {
    Push-Location -LiteralPath $pythonRoot
    try {
        & $Python -m game_inspection.compare.compare_runtime_builds `
            (Join-Path $OutputDirectory 'client') (Join-Path $OutputDirectory 'server') `
            --out (Join-Path $OutputDirectory 'client-to-server.json')
    } finally {
        Pop-Location
    }
    if ($LASTEXITCODE -ne 0) { throw 'Client/server comparison failed' }
}
Write-Output "Runtime discovery saved to $OutputDirectory"

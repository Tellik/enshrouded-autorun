param(
    [string]$ArchiveDirectory = 'build',
    [string]$ReportPath = 'defender-release-report.json'
)

$ErrorActionPreference = 'Stop'
$scanStartedAt = [DateTime]::UtcNow
$runId = if ($env:GITHUB_RUN_ID) { $env:GITHUB_RUN_ID } else { [Guid]::NewGuid().ToString('N') }
$temporaryRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
$scanRoot = Join-Path $temporaryRoot "shroudforge-release-scan-$runId"
$report = [ordered]@{
    scanner = 'Microsoft Defender Antivirus'
    target = 'Release ZIP files and extracted contents'
    status = 'error'
    scan_started_at = $scanStartedAt.ToString('o')
    scan_completed_at = $null
    archive_count = 0
    scanned_file_count = 0
    signature_update_status = 'not_attempted'
    signature_update_message = $null
    metadata = [ordered]@{}
    metadata_warnings = @()
    detection_count = 0
    detections = @()
    error = $null
}

function Get-ReleaseDetectionDetails {
    param([string[]]$ResourceTargets)

    $matchedDetections = @(Get-MpThreatDetection -ErrorAction Stop | Where-Object {
        $resourceText = (@($_.Resources) | ForEach-Object { [string]$_ }) -join "`n"
        $matchedTarget = $false
        foreach ($target in $ResourceTargets) {
            if ($resourceText.IndexOf($target, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
                $matchedTarget = $true
                break
            }
        }
        $matchedTarget
    })

    @($matchedDetections | ForEach-Object {
        [ordered]@{
            threat_name = [string]$_.ThreatName
            initial_detection_time = if ($_.InitialDetectionTime) { $_.InitialDetectionTime.ToUniversalTime().ToString('o') } else { $null }
            resources = @($_.Resources | ForEach-Object { [string]$_ })
        }
    })
}

try {
    try {
        Update-MpSignature -ErrorAction Stop
        $report.signature_update_status = 'success'
        Write-Host 'Microsoft Defender signature update completed.'
    } catch {
        $report.signature_update_status = 'warning'
        $report.signature_update_message = $_.Exception.Message
        Write-Warning "Microsoft Defender signature update failed. Scanning with the signatures already present. $($_.Exception.Message)"
    }

    try {
        $computerStatus = Get-MpComputerStatus -ErrorAction Stop
        $report.metadata.engine_version = [string]$computerStatus.AMEngineVersion
        $report.metadata.signature_version = [string]$computerStatus.AntivirusSignatureVersion
        $report.metadata.signature_last_updated = if ($computerStatus.AntivirusSignatureLastUpdated) { $computerStatus.AntivirusSignatureLastUpdated.ToUniversalTime().ToString('o') } else { $null }
        $report.metadata.real_time_protection_enabled = $computerStatus.RealTimeProtectionEnabled
        $preference = Get-MpPreference -ErrorAction Stop
        $report.metadata.cloud_protection_level = $preference.MAPSReporting
    } catch {
        $report.metadata_warnings += "Could not read Defender engine or protection settings: $($_.Exception.Message)"
    }

    $archives = @(Get-ChildItem -LiteralPath $ArchiveDirectory -Filter 'shroudforge-*.zip' -File)
    if ($archives.Count -eq 0) {
        throw "No release ZIPs were found in '$ArchiveDirectory'."
    }
    $report.archive_count = $archives.Count
    $report.archives = @($archives | ForEach-Object { $_.Name })
    $detectionTargets = @($scanRoot) + @($archives | ForEach-Object { $_.FullName })

    New-Item -ItemType Directory -Path $scanRoot -Force | Out-Null
    foreach ($archive in $archives) {
        $scanArchive = Join-Path $scanRoot $archive.Name
        Copy-Item -LiteralPath $archive.FullName -Destination $scanArchive
        $destination = Join-Path $scanRoot ([IO.Path]::GetFileNameWithoutExtension($archive.Name))
        Expand-Archive -LiteralPath $archive.FullName -DestinationPath $destination
    }

    $report.scanned_file_count = @(Get-ChildItem -LiteralPath $scanRoot -File -Recurse).Count
    Start-MpScan -ScanType CustomScan -ScanPath $scanRoot -ErrorAction Stop

    # The scan directory is unique to this workflow run. Include every detection
    # that references it or a downloaded ZIP, including real-time detections.
    $report.detections = @(Get-ReleaseDetectionDetails -ResourceTargets $detectionTargets)
    $report.detection_count = $report.detections.Count
    $report.scan_completed_at = [DateTime]::UtcNow.ToString('o')
    if ($report.detection_count -gt 0) {
        $report.status = 'findings'
        Write-Host "Microsoft Defender reported $($report.detection_count) detection(s) in the release contents."
    } else {
        $report.status = 'clean'
        Write-Host "Microsoft Defender found no malware in $($archives.Count) release ZIP(s)."
    }
} catch {
    $failureMessage = $_.Exception.Message
    $report.status = 'error'
    $report.error = $failureMessage
    $report.scan_completed_at = [DateTime]::UtcNow.ToString('o')
    if ($detectionTargets) {
        try {
            $report.detections = @(Get-ReleaseDetectionDetails -ResourceTargets $detectionTargets)
            $report.detection_count = $report.detections.Count
            if ($report.detection_count -gt 0) {
                $report.status = 'findings'
            }
        } catch {
            $report.metadata_warnings += "Could not retrieve Defender detections after the scan error: $($_.Exception.Message)"
        }
    }
    Write-Error "Microsoft Defender release scan failed: $failureMessage" -ErrorAction Continue
} finally {
    $reportDirectory = Split-Path -Parent $ReportPath
    if ($reportDirectory) {
        New-Item -ItemType Directory -Path $reportDirectory -Force | Out-Null
    }
    $report | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $ReportPath -Encoding utf8NoBOM
}

if ($report.status -ne 'clean') {
    exit 1
}

param(
    [string]$Category,
    [string]$OutputPath = "build/nexusmods/enshrouded-latest-code-files.tree.txt",
    [int]$PageSize = 100,
    [int]$DelayMilliseconds = 250,
    [int]$MaxMods = 0
)

$ErrorActionPreference = "Stop"

if ($PageSize -lt 1 -or $PageSize -gt 500) {
    throw "PageSize must be between 1 and 500."
}
if ($DelayMilliseconds -lt 0) {
    throw "DelayMilliseconds cannot be negative."
}
if ($MaxMods -lt 0) {
    throw "MaxMods cannot be negative. Use 0 for all matching mods."
}

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if ([System.IO.Path]::IsPathRooted($OutputPath)) {
    $resolvedOutputPath = $OutputPath
} else {
    $resolvedOutputPath = Join-Path $repositoryRoot $OutputPath
}
$apiKey = $env:NEXUS_API_KEY
$endpoint = "https://api.nexusmods.com/v2/graphql"
$restEndpoint = "https://api.nexusmods.com/v1"
$applicationVersion = (Get-Content -Raw (Join-Path $repositoryRoot "VERSION")).Trim()

if ([string]::IsNullOrWhiteSpace($apiKey)) {
    throw "Set the NEXUS_API_KEY environment variable. Nexus REST v1 is required to read content_preview_link for current archive previews."
}

$modsQuery = @'
query EnshroudedMods($filter: ModsFilter, $offset: Int, $count: Int) {
  mods(filter: $filter, offset: $offset, count: $count) {
    nodes {
      gameId
      modId
      name
      category
      version
      updatedAt
    }
    nodesCount
    totalCount
  }
}
'@

$filter = @{
    op = "AND"
    gameDomainName = @(@{ value = "enshrouded"; op = "EQUALS" })
}
if (-not [string]::IsNullOrWhiteSpace($Category)) {
    $filter.categoryName = @(@{ value = $Category.Trim(); op = "EQUALS" })
}

$headers = @{
    Accept = "application/json"
    "Application-Name" = "ShroudForge dev catalog exporter"
    "Application-Version" = $applicationVersion
}
if (-not [string]::IsNullOrWhiteSpace($apiKey)) {
    $headers.apikey = $apiKey
}

function Invoke-NexusGraphQl {
    param(
        [Parameter(Mandatory = $true)][string]$Query,
        [Parameter(Mandatory = $true)][hashtable]$Variables
    )

    $body = @{
        query = $Query
        variables = $Variables
    } | ConvertTo-Json -Depth 20

    $response = Invoke-RestMethod `
        -Method Post `
        -Uri $endpoint `
        -Headers $headers `
        -ContentType "application/json" `
        -Body $body

    if ($response.errors) {
        $messages = ($response.errors | ForEach-Object { $_.message }) -join " | "
        throw "Nexus Mods GraphQL request failed: $messages"
    }
    return $response.data
}

function Get-NexusModFiles {
    param([Parameter(Mandatory = $true)][int]$ModId)

    $uri = "$restEndpoint/games/enshrouded/mods/$ModId/files.json"
    $response = Invoke-RestMethod `
        -Method Get `
        -Uri $uri `
        -Headers $headers

    if ($null -eq $response.files) {
        throw "Nexus REST v1 response did not contain files for mod $ModId."
    }
    return @($response.files)
}

function Join-ArchivePath {
    param(
        [AllowEmptyString()][string]$ParentPath,
        [AllowEmptyString()][string]$ChildPath
    )

    $child = $ChildPath -replace "\\", "/"
    if ([string]::IsNullOrWhiteSpace($child)) {
        return $ParentPath
    }
    if ($child.StartsWith("/") -or $child -match "^[A-Za-z]:") {
        throw "Nexus preview contained an absolute archive path: $ChildPath"
    }

    $combined = if ([string]::IsNullOrWhiteSpace($ParentPath)) { $child } else { "$ParentPath/$child" }
    $parts = @($combined.Split("/") | Where-Object { $_ -and $_ -ne "." })
    if ($parts -contains "..") {
        throw "Nexus preview contained a parent traversal path: $ChildPath"
    }
    return ($parts -join "/")
}

function Add-PreviewFiles {
    param(
        [Parameter(Mandatory = $true)]$Node,
        [Parameter(Mandatory = $true)][AllowEmptyString()][string]$ParentPath,
        [Parameter(Mandatory = $true)][AllowEmptyCollection()][System.Collections.Generic.List[string]]$Paths
    )

    if ($null -eq $Node) {
        return
    }
    if ($Node -is [System.Collections.IEnumerable] -and $Node -isnot [string]) {
        foreach ($item in $Node) {
            Add-PreviewFiles -Node $item -ParentPath $ParentPath -Paths $Paths
        }
        return
    }

    $properties = $Node.PSObject.Properties
    $type = [string]$Node.type
    $name = [string]$Node.name
    $path = [string]$Node.path
    $filePath = [string]$Node.file_path
    $children = if ($properties["children"]) { $Node.children } else { $null }
    $files = if ($properties["files"]) { $Node.files } else { $null }

    if ($type -eq "file" -or -not [string]::IsNullOrWhiteSpace($filePath) -or ($null -eq $children -and $null -eq $files -and -not [string]::IsNullOrWhiteSpace($path))) {
        $previewPath = if (-not [string]::IsNullOrWhiteSpace($filePath)) { $filePath } else { $path }
        $relativePath = if ([string]::IsNullOrWhiteSpace($previewPath)) { Join-ArchivePath -ParentPath $ParentPath -ChildPath $name } else { Join-ArchivePath -ParentPath "" -ChildPath $previewPath }
        if (-not [string]::IsNullOrWhiteSpace($relativePath)) {
            $Paths.Add($relativePath)
        }
        return
    }

    $directoryPath = if (-not [string]::IsNullOrWhiteSpace($path)) {
        Join-ArchivePath -ParentPath "" -ChildPath $path
    } elseif (-not [string]::IsNullOrWhiteSpace($name)) {
        Join-ArchivePath -ParentPath $ParentPath -ChildPath $name
    } else {
        $ParentPath
    }

    if ($null -ne $children) {
        Add-PreviewFiles -Node $children -ParentPath $directoryPath -Paths $Paths
    }
    if ($null -ne $files) {
        Add-PreviewFiles -Node $files -ParentPath $ParentPath -Paths $Paths
    }
}

function Get-NexusPreviewPaths {
    param([Parameter(Mandatory = $true)][string]$PreviewUri)

    if ([string]::IsNullOrWhiteSpace($PreviewUri)) {
        throw "Nexus file metadata did not provide content_preview_link."
    }
    $previewUrl = [Uri]$PreviewUri
    if ($previewUrl.Scheme -ne "https") {
        throw "Nexus content preview URL must use HTTPS: $PreviewUri"
    }
    if ($previewUrl.UserInfo -or $previewUrl.Fragment) {
        throw "Nexus content preview URL must not contain credentials or a fragment."
    }
    if ($previewUrl.Host -eq "file-metadata.nexusmods.com") {
        $segments = @($previewUrl.AbsolutePath.Split("/"))
        if ($segments.Count -ge 6 -and $segments[1] -eq "file" -and $segments[2] -eq "nexus-files-s3-meta" -and $segments[3] -match "^\d+$" -and $segments[4] -match "^\d+$" -and $segments[-1].EndsWith(".json")) {
            $manifestParts = @($segments[5..($segments.Count - 1)])
            $manifestParts[-1] = $manifestParts[-1].Substring(0, $manifestParts[-1].Length - 5)
            if (@($manifestParts | Where-Object { $_ -notmatch "^[0-9a-f-]+$" }).Count -eq 0) {
                $manifestPath = ($manifestParts -join "/")
                $previewUrl = [Uri]("https://mod-file-manifests.nexusmods.com/$manifestPath")
            }
        }
    }
    if ($previewUrl.Host -notin @("file-metadata.nexusmods.com", "mod-file-manifests.nexusmods.com")) {
        throw "Nexus returned an unrecognized content preview host: $($previewUrl.Host)"
    }

    # Preview JSON is served by Nexus metadata CDN. Do not forward the Nexus API key to it.
    $manifest = Invoke-RestMethod -Method Get -Uri $previewUrl.AbsoluteUri
    $paths = [System.Collections.Generic.List[string]]::new()
    Add-PreviewFiles -Node $manifest -ParentPath "" -Paths $paths
    if ($paths.Count -eq 0) {
        throw "Nexus content preview JSON contained no recognized file paths."
    }
    return $paths
}

function New-TreeNode {
    return [pscustomobject]@{
        Children = [ordered]@{}
        IsFile = $false
    }
}

function Add-TreePath {
    param(
        [Parameter(Mandatory = $true)]$Root,
        [Parameter(Mandatory = $true)][string[]]$Parts
    )

    $current = $Root
    foreach ($part in $Parts) {
        if ([string]::IsNullOrWhiteSpace($part) -or $part -eq ".") {
            continue
        }
        if ($part -eq "..") {
            continue
        }
        if (-not $current.Children.Contains($part)) {
            $current.Children[$part] = New-TreeNode
        }
        $current = $current.Children[$part]
    }
    $current.IsFile = $true
}

function Write-TreeLines {
    param(
        [Parameter(Mandatory = $true)]$Node,
        [Parameter(Mandatory = $true)][AllowEmptyString()][string]$Prefix,
        [Parameter(Mandatory = $true)][System.Collections.Generic.List[string]]$Lines
    )

    $children = @($Node.Children.Keys | Sort-Object)
    for ($index = 0; $index -lt $children.Count; $index++) {
        $name = [string]$children[$index]
        $isLast = $index -eq ($children.Count - 1)
        $branch = if ($isLast) { "└── " } else { "├── " }
        $child = $Node.Children[$name]
        $suffix = if ($child.IsFile) { " [Datei]" } else { "/" }
        $Lines.Add("$Prefix$branch$name$suffix")
        if ($child.Children.Count -gt 0) {
            $nextPrefix = if ($isLast) { "$Prefix    " } else { "$Prefix│   " }
            Write-TreeLines -Node $child -Prefix $nextPrefix -Lines $Lines
        }
    }
}

$mods = [System.Collections.Generic.List[object]]::new()
$offset = 0
$totalCount = $null
do {
    $pageData = Invoke-NexusGraphQl -Query $modsQuery -Variables @{
        filter = $filter
        offset = $offset
        count = $PageSize
    }
    $page = $pageData.mods
    if ($null -eq $page) {
        throw "Nexus Mods GraphQL response did not contain data.mods."
    }
    if ($null -eq $totalCount) {
        $totalCount = [int]$page.totalCount
    }
    foreach ($mod in $page.nodes) {
        $mods.Add($mod)
    }
    $returnedCount = @($page.nodes).Count
    if ($returnedCount -eq 0) {
        break
    }
    $offset += $returnedCount
    Write-Host "Enshrouded-Mods gefunden: $offset von $totalCount"
} while ($offset -lt $totalCount)

if ($MaxMods -gt 0 -and $mods.Count -gt $MaxMods) {
    Write-Warning "Auf $MaxMods Mods begrenzt. Für alle Treffer erneut mit -MaxMods 0 starten."
    $limitedMods = [System.Collections.Generic.List[object]]::new()
    for ($index = 0; $index -lt $MaxMods; $index++) {
        $limitedMods.Add($mods[$index])
    }
    $mods = $limitedMods
}

$treeLines = [System.Collections.Generic.List[string]]::new()
$treeLines.Add("Enshrouded Nexus Mods: alle Archivdateien aus den Datei-Previews")
$treeLines.Add("Erstellt (UTC): $([DateTime]::UtcNow.ToString('o'))")
$treeLines.Add("Mods: $($mods.Count)")
$treeLines.Add("Hinweis: Dateipfade stammen aus Nexus-Metadaten. Es wurden keine Archive heruntergeladen.")
$treeLines.Add("---")

$modIndex = 0
foreach ($mod in $mods) {
    $modIndex++
    if ($DelayMilliseconds -gt 0) {
        Start-Sleep -Milliseconds $DelayMilliseconds
    }

    $allFiles = Get-NexusModFiles -ModId ([int]$mod.modId)
    $currentFiles = @(
        $allFiles |
            Where-Object { $_.category_name -in @("MAIN", "UPDATE", "OPTIONAL", "MISCELLANEOUS") } |
            Sort-Object -Property category_name, @{ Expression = { [long]$_.uploaded_timestamp }; Descending = $true }
    )
    $modHeader = "[$modIndex/$($mods.Count)] $($mod.name) (ID $($mod.modId))"
    if ($currentFiles.Count -eq 0) {
        $treeLines.Add($modHeader)
        $treeLines.Add("└── Keine aktuelle Moddatei in der API gelistet")
        $treeLines.Add("---")
        continue
    }

    $treeLines.Add($modHeader)
    for ($fileIndex = 0; $fileIndex -lt $currentFiles.Count; $fileIndex++) {
        $currentFile = $currentFiles[$fileIndex]
        $isLastFile = $fileIndex -eq ($currentFiles.Count - 1)
        $fileBranch = if ($isLastFile) { "└── " } else { "├── " }
        $filePrefix = if ($isLastFile) { "    " } else { "│   " }
        $treeLines.Add("$fileBranch[$($currentFile.category_name)] $($currentFile.name), Version $($currentFile.version), Datei-ID $($currentFile.file_id)")
        $fileTree = New-TreeNode
        $previewUri = [string]$currentFile.content_preview_link
        try {
            $previewPaths = Get-NexusPreviewPaths -PreviewUri $previewUri
        } catch {
            $treeLines[$treeLines.Count - 1] = "$fileBranch[$($currentFile.category_name)] $($currentFile.name), Version $($currentFile.version), Datei-ID $($currentFile.file_id), Vorschaufehler"
            $treeLines.Add("$filePrefix└── Vorschau konnte nicht gelesen werden: $($_.Exception.Message)")
            continue
        }
        foreach ($path in $previewPaths) {
            $parts = @($path.Split("/") | Where-Object { $_ })
            Add-TreePath -Root $fileTree -Parts $parts
        }
        $treeLines[$treeLines.Count - 1] = "$fileBranch[$($currentFile.category_name)] $($currentFile.name), Version $($currentFile.version), Datei-ID $($currentFile.file_id), Vorschau $($previewPaths.Count) Archivdateien"

        if ($previewPaths.Count -eq 0) {
            $treeLines.Add("$filePrefix└── Nexus hat für diese Datei keine Vorschaupfade bereitgestellt")
        } elseif ($fileTree.Children.Count -eq 0) {
            $treeLines.Add("$filePrefix└── Nexus-Dateipfade konnten nicht als Baum dargestellt werden")
        } else {
            Write-TreeLines -Node $fileTree -Prefix $filePrefix -Lines $treeLines
        }
    }
    $treeLines.Add("---")
    Write-Host "Dateibaum erstellt: $modHeader"
}

$outputDirectory = Split-Path -Parent $resolvedOutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}
[System.IO.File]::WriteAllLines($resolvedOutputPath, $treeLines, [System.Text.UTF8Encoding]::new($false))

Write-Host "Dateibaum gespeichert: $resolvedOutputPath"
Write-Host "Es wurden keine Moddateien heruntergeladen."

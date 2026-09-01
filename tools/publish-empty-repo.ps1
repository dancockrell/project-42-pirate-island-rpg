param(
    [Parameter(Mandatory = $true)]
    [string]$Repository,

    [string]$Branch = "main",

    [string]$Message = "Initialize Project 42 vertical slice foundation"
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$gh = "C:\Program Files\GitHub CLI\gh.exe"
$env:GH_CONFIG_DIR = Join-Path $workspace ".gh-auth"

function Invoke-GitHubJson {
    param(
        [Parameter(Mandatory = $true)] [string]$Endpoint,
        [Parameter(Mandatory = $true)] [object]$Body,
        [string]$Method = "POST"
    )

    $json = $Body | ConvertTo-Json -Depth 12 -Compress
    $response = $json | & $gh api --method $Method $Endpoint --input -
    if ($LASTEXITCODE -ne 0) {
        throw "GitHub API request failed: $Endpoint"
    }
    return $response | ConvertFrom-Json
}

$gitDirectoryArgument = "--git-dir=$(Join-Path $workspace '.git-meta')"
$workTreeArgument = "--work-tree=$workspace"
$trackedFiles = & git $gitDirectoryArgument $workTreeArgument ls-files
if ($LASTEXITCODE -ne 0) {
    throw "Unable to list tracked files."
}

$treeEntries = foreach ($relativePath in $trackedFiles) {
    $absolutePath = Join-Path $workspace $relativePath
    $bytes = [IO.File]::ReadAllBytes($absolutePath)
    $blob = Invoke-GitHubJson -Endpoint "repos/$Repository/git/blobs" -Body @{
        content = [Convert]::ToBase64String($bytes)
        encoding = "base64"
    }
    [ordered]@{
        path = $relativePath.Replace("\", "/")
        mode = "100644"
        type = "blob"
        sha = $blob.sha
    }
    Write-Host "Uploaded $relativePath"
}

$tree = Invoke-GitHubJson -Endpoint "repos/$Repository/git/trees" -Body @{
    tree = @($treeEntries)
}

$currentReferenceJson = & $gh api "repos/$Repository/git/ref/heads/$Branch" 2>$null
$currentCommit = if ($LASTEXITCODE -eq 0) { ($currentReferenceJson | ConvertFrom-Json).object.sha } else { $null }
$parents = [object[]]@()
if ($currentCommit) {
    $parents = [object[]]@($currentCommit)
}

$commit = Invoke-GitHubJson -Endpoint "repos/$Repository/git/commits" -Body @{
    message = $Message
    tree = $tree.sha
    parents = $parents
}

if ($currentCommit) {
    $reference = Invoke-GitHubJson -Endpoint "repos/$Repository/git/refs/heads/$Branch" -Method "PATCH" -Body @{
        sha = $commit.sha
        force = $false
    }
} else {
    $reference = Invoke-GitHubJson -Endpoint "repos/$Repository/git/refs" -Body @{
        ref = "refs/heads/$Branch"
        sha = $commit.sha
    }
}

Write-Host "Published $Repository@$Branch at $($commit.sha)"

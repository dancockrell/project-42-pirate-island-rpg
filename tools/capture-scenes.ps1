# The visual loop on Windows. Portable twin of tools/capture-scenes.sh: the two
# scripts render the same scenes, fail on the same conditions and print the same
# messages; only the way Godot is located and the way its per-user directories
# are redirected differ per platform.
#
# Two engine runs, and never more: one import pass, then ONE process that
# renders every scene, for the reason capture-scenes.sh gives.
#
# Renders every scene under game/scenes/review, game/scenes/world,
# game/scenes/battle and game/scenes/shell to work/captures/<scene>.png at
# 1920x1080, with a 1280x720 <scene>.thumb.png beside it.
param(
    [string]$GodotExecutable = "",
    [string]$OutputDirectory = "",
    [int]$Width = 1920,
    [int]$Height = 1080,
    [int]$ThumbnailWidth = 1280,
    [int]$ThumbnailHeight = 720,
    [int]$Frames = 6
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($GodotExecutable)) {
    $GodotExecutable = Join-Path $workspace ".local-tools\godot-4.7.2\Godot_v4.7.2-stable_win64_console.exe"
}
if (-not (Test-Path -LiteralPath $GodotExecutable -PathType Leaf)) {
    throw "Godot executable not found: $GodotExecutable"
}
$gamePath = Join-Path $workspace "game"
if (-not (Test-Path -LiteralPath (Join-Path $gamePath "project.godot") -PathType Leaf)) {
    throw "Godot project not found: $(Join-Path $gamePath 'project.godot')"
}
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path $workspace "work\captures"
}
New-Item -ItemType Directory -Force $OutputDirectory | Out-Null

# The driver: Forward+ under Vulkan when a device is actually present, the
# OpenGL compatibility renderer otherwise, which is what game/project.godot
# declares as the project's method. Which one ran is printed, and it belongs in
# any judgement of the images: the two do not light a scene identically.
$renderingDriver = "opengl3"
$driverReason = "no Vulkan device reported by vulkaninfo"
$vulkanInfo = Get-Command vulkaninfo -ErrorAction SilentlyContinue
if ($null -eq $vulkanInfo) {
    $driverReason = "vulkaninfo is not installed"
} else {
    & $vulkanInfo.Source --summary *> $null
    if ($LASTEXITCODE -eq 0) {
        $renderingDriver = "vulkan"
        $driverReason = "a Vulkan device is present"
    }
}
Write-Host "==> Rendering driver: $renderingDriver ($driverReason)"
Write-Host "==> Output directory: $OutputDirectory"

# Same rule as tools/verify-godot.ps1 and tools/capture-scenes.sh: a step that
# exits 0 but prints an ERROR line has failed.
$errorPattern = '^(ERROR|SCRIPT ERROR): |GDScript backtrace'

$taskProfile = Join-Path $env:TEMP "project42-godot-profile"
$taskLocal = Join-Path $env:TEMP "project42-godot-local"
New-Item -ItemType Directory -Force $taskProfile, $taskLocal | Out-Null
$previousAppData = $env:APPDATA
$previousLocalAppData = $env:LOCALAPPDATA
$written = @()
try {
    $env:APPDATA = $taskProfile
    $env:LOCALAPPDATA = $taskLocal

    # The editor import pass, exactly as verify-godot.ps1 runs it. Without it
    # the global script class cache is empty on a fresh checkout and every
    # scene whose script names a class_name fails to parse.
    Write-Host "==> Godot import and extension registration"
    & $GodotExecutable --headless --editor --path $gamePath --quit
    if ($LASTEXITCODE -ne 0) {
        throw "Godot import and extension registration failed with exit code $LASTEXITCODE"
    }

    # Enumerated rather than listed, so a new scene is captured the day it
    # lands. A directory that does not exist yet -- scenes/shell, until the
    # shell card builds it -- is skipped rather than failing the run.
    $scenes = @()
    foreach ($directory in @("review", "world", "battle", "shell")) {
        $root = Join-Path $gamePath "scenes\$directory"
        if (-not (Test-Path -LiteralPath $root -PathType Container)) { continue }
        $scenes += Get-ChildItem -LiteralPath $root -Recurse -Filter *.tscn -File | Sort-Object FullName
    }
    if ($scenes.Count -eq 0) {
        throw "No scenes found under $gamePath\scenes\{review,world,battle,shell}\"
    }
    $scenePaths = $scenes | ForEach-Object {
        "res://" + $_.FullName.Substring($gamePath.Length + 1).Replace("\", "/")
    }
    Write-Host "==> $($scenePaths.Count) scenes, one engine process"

    # One process for every scene, never one per scene: an engine start costs
    # about ten seconds and a great deal of memory. --audio-driver Dummy
    # because a capture is silent, and a run on a machine with no sound device
    # otherwise prints audio failures that the ERROR check would read as a
    # broken scene.
    $arguments = @(
        "--rendering-driver", $renderingDriver,
        "--audio-driver", "Dummy",
        "--path", $gamePath,
        "--script", "res://tools/capture_review_scene.gd",
        "--"
    ) + $scenePaths + @(
        "--out-dir", $OutputDirectory,
        "--thumbnails",
        "--frames", "$Frames",
        "--size", "${Width}x${Height}",
        "--thumbnail-size", "${ThumbnailWidth}x${ThumbnailHeight}"
    )
    $runOutput = & $GodotExecutable @arguments 2>&1 | ForEach-Object { "$_" }
    $runOutput | ForEach-Object { Write-Host $_ }
    if ($LASTEXITCODE -ne 0) {
        throw "The capture run failed with exit code $LASTEXITCODE"
    }
    if ($runOutput | Where-Object { $_ -match $errorPattern }) {
        throw "The capture run exited 0 but printed an ERROR line; an exit code is not the only verdict"
    }

    # A green run that wrote nothing is the exact failure this gate exists to
    # make impossible, so the files are counted on disk rather than taken on
    # trust.
    $written = @(Get-ChildItem -LiteralPath $OutputDirectory -Filter *.png -File | Where-Object { $_.Length -gt 0 })
    if ($written.Count -eq 0) {
        throw "The capture run reported success but left no PNG in $OutputDirectory"
    }
} finally {
    $env:APPDATA = $previousAppData
    $env:LOCALAPPDATA = $previousLocalAppData
}
Write-Host "Captures complete: $($written.Count) files in $OutputDirectory on the $renderingDriver driver."

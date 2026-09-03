param(
    [ValidateSet("debug", "release")]
    [string]$Configuration = "debug"
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $workspace "godot-rust\Cargo.toml"
$arguments = @("build", "--manifest-path", $manifest, "--features", "godot-ext")
if ($Configuration -eq "release") { $arguments += "--release" }

& cargo @arguments
if ($LASTEXITCODE -ne 0) { throw "Rust GDExtension build failed with exit code $LASTEXITCODE" }

$profile = if ($Configuration -eq "release") { "release" } else { "debug" }
$source = Join-Path $workspace "godot-rust\target\$profile\project42_sim.dll"
$destinationDirectory = Join-Path $workspace "game\bin\windows"
$destinationName = if ($Configuration -eq "release") { "project42_sim.windows.expedition_v4_release.x86_64.dll" } else { "project42_sim.windows.expedition_v4_debug.x86_64.dll" }
$destination = Join-Path $destinationDirectory $destinationName

if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { throw "Expected native library was not produced: $source" }
New-Item -ItemType Directory -Force $destinationDirectory | Out-Null
Copy-Item -LiteralPath $source -Destination $destination -Force
Write-Host "Built native bridge: $destination"

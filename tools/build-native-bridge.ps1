# Windows twin of tools/build-native-bridge.sh. Same sequence, same failure
# behaviour, and -- E11 -- the same rule for the installed file name: it carries
# the architecture this build actually produced rather than a hard-coded x86_64,
# because Godot resolves windows.<config>.<arch of the host it is running on>
# and a name that lies about the architecture is a build with no simulation in
# it. The bash twin reads `uname -m`; here the same fact comes from .NET's
# OSArchitecture, mapped to Godot's tokens.

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

$osArchitecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
$architecture = switch ($osArchitecture) {
    "X64"   { "x86_64" }
    "Arm64" { "arm64" }
    default { throw "Unsupported architecture for the native bridge build: $osArchitecture" }
}

$profile = if ($Configuration -eq "release") { "release" } else { "debug" }
$source = Join-Path $workspace "godot-rust\target\$profile\project42_sim.dll"
$destinationDirectory = Join-Path $workspace "game\bin\windows"
$destinationName = "project42_sim.windows.expedition_v5_${Configuration}.${architecture}.dll"
$destination = Join-Path $destinationDirectory $destinationName

if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { throw "Expected native library was not produced: $source" }
New-Item -ItemType Directory -Force $destinationDirectory | Out-Null
Copy-Item -LiteralPath $source -Destination $destination -Force
Write-Host "Built native bridge: $destination"

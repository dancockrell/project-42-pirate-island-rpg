param(
    [string]$GodotExecutable = "",
    [ValidateRange(1, 60)]
    [int]$TimeoutSeconds = 30
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($GodotExecutable)) {
    $GodotExecutable = Join-Path $workspace ".local-tools\godot-4.7.2\Godot_v4.7.2-stable_win64_console.exe"
}
if (-not (Test-Path -LiteralPath $GodotExecutable -PathType Leaf)) {
    throw "Godot executable not found: $GodotExecutable"
}

# Editor import pass first: a fresh checkout has no .godot cache, and without it
# a script that preloads another by path fails to resolve. The pass registers
# the extension and imports resources, then quits.
$importArguments = @("--headless", "--editor", "--path", (Join-Path $workspace "game"), "--quit")
& (Resolve-Path -LiteralPath $GodotExecutable).Path @importArguments *> $null
if ($LASTEXITCODE -ne 0) { throw "Godot import and extension registration failed with exit code $LASTEXITCODE" }

# Verify the current sprite-based island.
$checks = @(
    @{ Script = "res://tests/directional_sprite_test.gd"; Completion = "PASS: four facings, alpha-source identity, frame regions, nearest sampling, idle retention, no simulation movement" },
    @{ Script = "res://tests/island_scene_test.gd"; Completion = "PASS: island scene suite complete" },
    @{ Script = "res://tests/island_bridge_test.gd"; Completion = "PASS: actual Rust island bridge, scenario document, solo start, movement, ocean rejection, pause and arrival" }
)
foreach ($check in $checks) {
    $start = [System.Diagnostics.ProcessStartInfo]::new()
    $start.FileName = (Resolve-Path -LiteralPath $GodotExecutable).Path
    $start.WorkingDirectory = $workspace
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    foreach ($argument in @("--headless", "--path", (Join-Path $workspace "game"), "--script", $check.Script, "--quit-after", "120")) {
        $start.ArgumentList.Add($argument)
    }
    # Unique profiles avoid concurrent tests sharing or overwriting user saves.
    $profile = Join-Path ([System.IO.Path]::GetTempPath()) ("project42-test-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $profile | Out-Null
    $start.Environment["APPDATA"] = $profile
    $start.Environment["LOCALAPPDATA"] = $profile
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $start
    try {
        if (-not $process.Start()) { throw "Could not start bounded Godot test." }
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
            $process.Kill($true)
            $process.WaitForExit()
            throw "Godot test timed out after $TimeoutSeconds seconds: $($check.Script)"
        }
        $output = $stdout.GetAwaiter().GetResult() + $stderr.GetAwaiter().GetResult()
        Write-Host $output
        if ($process.ExitCode -ne 0 -or $output -match '(?m)^(SCRIPT ERROR:|ERROR:|USER ERROR:)') {
            throw "Godot test failed: $($check.Script) (exit $($process.ExitCode))"
        }
        if (-not ($output -split '\r?\n').Contains($check.Completion)) {
            throw "Godot exited without completing test: $($check.Script)"
        }
    } finally {
        $process.Dispose()
        # Keep isolated diagnostics for inspection; never prune shared user data.
        Write-Host "Isolated test profile: $profile"
    }
}
Write-Host "Current 2D Godot verification passed (not rendered visual acceptance)."

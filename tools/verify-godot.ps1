param(
    [string]$GodotExecutable = ""
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($GodotExecutable)) {
    $GodotExecutable = Join-Path $workspace ".local-tools\godot-4.7.2\Godot_v4.7.2-stable_win64_console.exe"
}
if (-not (Test-Path -LiteralPath $GodotExecutable -PathType Leaf)) {
    throw "Godot executable not found: $GodotExecutable"
}

$taskProfile = Join-Path $env:TEMP "project42-godot-profile"
$taskLocal = Join-Path $env:TEMP "project42-godot-local"
New-Item -ItemType Directory -Force $taskProfile, $taskLocal | Out-Null
$previousAppData = $env:APPDATA
$previousLocalAppData = $env:LOCALAPPDATA
try {
    $env:APPDATA = $taskProfile
    $env:LOCALAPPDATA = $taskLocal
    & $GodotExecutable --headless --editor --path (Join-Path $workspace "game") --quit
    if ($LASTEXITCODE -ne 0) {
        throw "Godot import and extension registration failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --quit-after 3
    if ($LASTEXITCODE -ne 0) {
        throw "Godot headless verification failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --scene "res://scenes/review/betty_3d_candidate_review.tscn" --quit-after 2
    if ($LASTEXITCODE -ne 0) {
        throw "Godot Betty 3D candidate review scene failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/battle_3d_staging_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot 3D battle staging tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/world_cell_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot world-cell tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --scene "res://scenes/review/reception_terrace_setpiece_review.tscn" --quit-after 2
    if ($LASTEXITCODE -ne 0) {
        throw "Godot Reception Terrace setpiece review scene failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/reception_terrace_setpiece_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot Reception Terrace setpiece tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/targeting_session_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot targeting-session tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/skill_animation_director_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot skill-animation-director tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/placeholder_action_presenter_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot placeholder-action-presenter tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/paper_betty_rig_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot paper-Betty rig tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/content_catalog_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot content-catalog registry tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/combat_text_renderer_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot combat-text-renderer tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/native_simulation_port_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot native-simulation-port tests failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/prototype_turn_cycle_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot battle-prototype turn-cycle test failed with exit code $LASTEXITCODE"
    }
    & $GodotExecutable --headless --path (Join-Path $workspace "game") --script "res://tests/betty_3d_candidate_contract_test.gd"
    if ($LASTEXITCODE -ne 0) {
        throw "Godot Betty 3D candidate contract test failed with exit code $LASTEXITCODE"
    }
} finally {
    $env:APPDATA = $previousAppData
    $env:LOCALAPPDATA = $previousLocalAppData
}
Write-Host "Godot headless verification passed."

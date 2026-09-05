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

# Same rule as tools/verify-godot.sh: a step that exits 0 but prints an ERROR
# line has failed. SceneTree.quit() sets the exit code and returns, so a suite
# whose last statement is quit(0) reports success whatever its checks said.
$errorPattern = '^(ERROR|SCRIPT ERROR): |GDScript backtrace'
function Invoke-GodotStep {
    param(
        [string]$Description,
        [string[]]$Arguments
    )
    $output = & $GodotExecutable @Arguments 2>&1 | ForEach-Object { "$_" }
    $output | ForEach-Object { Write-Host $_ }
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed with exit code $LASTEXITCODE"
    }
    if ($output | Where-Object { $_ -match $errorPattern }) {
        throw "$Description exited 0 but printed an ERROR line; a suite's exit code is not the only verdict"
    }
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
    Invoke-GodotStep -Description "Godot headless verification" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --quit-after, 3)
    Invoke-GodotStep -Description "Godot Betty 3D candidate review scene" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --scene, "res://scenes/review/betty_3d_candidate_review.tscn", --quit-after, 2)
    Invoke-GodotStep -Description "Godot 3D battle staging tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/battle_3d_staging_test.gd")
    Invoke-GodotStep -Description "Godot world-cell tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/world_cell_test.gd")
    Invoke-GodotStep -Description "Godot Black Beach route contract tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/black_beach_route_contract_test.gd")
    Invoke-GodotStep -Description "Godot expedition-prototype tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/expedition_prototype_test.gd")
    Invoke-GodotStep -Description "Godot campaign-encounter-port tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/campaign_encounter_port_test.gd")
    Invoke-GodotStep -Description "Godot Reception Terrace setpiece review scene" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --scene, "res://scenes/review/reception_terrace_setpiece_review.tscn", --quit-after, 2)
    Invoke-GodotStep -Description "Godot Reception Terrace setpiece tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/reception_terrace_setpiece_test.gd")
    Invoke-GodotStep -Description "Godot Elizabethan port-town review scene" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --scene, "res://scenes/review/elizabethan_port_town_set_review.tscn", --quit-after, 2)
    Invoke-GodotStep -Description "Godot Elizabethan port-town contract test" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/elizabethan_port_town_set_test.gd")
    Invoke-GodotStep -Description "Godot targeting-session tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/targeting_session_test.gd")
    Invoke-GodotStep -Description "Godot skill-animation-director tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/skill_animation_director_test.gd")
    Invoke-GodotStep -Description "Godot placeholder-action-presenter tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/placeholder_action_presenter_test.gd")
    Invoke-GodotStep -Description "Godot paper-Betty rig tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/paper_betty_rig_test.gd")
    Invoke-GodotStep -Description "Godot paper-Razorbeak rig tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/paper_razorbeak_rig_test.gd")
    Invoke-GodotStep -Description "Godot content-catalog registry tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/content_catalog_test.gd")
    Invoke-GodotStep -Description "Godot combat-text-renderer tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/combat_text_renderer_test.gd")
    Invoke-GodotStep -Description "Godot native-simulation-port tests" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/native_simulation_port_test.gd")
    Invoke-GodotStep -Description "Godot battle-prototype turn-cycle test" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/prototype_turn_cycle_test.gd")
    Invoke-GodotStep -Description "Godot Betty 3D candidate contract test" -Arguments @(--headless, --path, (Join-Path $workspace "game"), --script, "res://tests/betty_3d_candidate_contract_test.gd")
} finally {
    $env:APPDATA = $previousAppData
    $env:LOCALAPPDATA = $previousLocalAppData
}
Write-Host "Godot headless verification passed."

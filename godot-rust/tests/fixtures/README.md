# Equality fixtures

`hard_coded_island_tick_0.json` and `hard_coded_island_tick_1440.json` are the
saves the deleted hard-coded island produced — `FactionWorld::prototype_island()`
followed by `install_preview_factions()`, at tick zero and after 1,440 ticks —
generated from this repository at commit `1e72809` before those two functions
were removed. They are version-1 saves, so they are also the fixture the
version-1 migration test resumes.

`scenario_world_equals_the_deleted_hard_coded_island` and
`version_one_save_migrates_into_its_scenario_and_plays_identically` in
`src/world.rs` are the tests that read them. Never regenerate them to make a
test pass: they are the record of what the simulation did before the scenario
pack replaced the compiled-in island.

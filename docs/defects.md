# Known Defects (current implementation)

Factual defects in the current code, used as a work queue. Not a changelog.
Resolved items are removed; history lives in `openspec/changes/archive/`.

## Open

1. `block-steel.PNG` filename case mismatch — `block.ron` references
   `block/block-steel.png`, but the file on disk is `block-steel.PNG`. Works on
   Windows (case-insensitive FS), breaks on case-sensitive filesystems.
   Fix: rename the file to `block-steel.png`.
2. Parallax block (`PullingBeam` turret) and the Colaris Turret share
   `colaris.png` — no distinct art exists for the beam weapon.
3. `find_schematic` (`mission/state.rs`) silently falls back to the first
   available design when a mission references a missing `schematic_name` — a
   typo in a mission `.ron` spawns the wrong design with no error. The player
   slot documents this fallback (see mission.md); enemy slots have no such
   contract and should fail loudly or log.
4. `ShipBehavior` components are never removed — AI re-issues every think
   tick; behavior flips are invisible on screen. Accepted.
5. `handle_block_destroy` may write duplicate `DestroyBlock` events for
   multi-step accumulations — rare; duplicate drops for non-players. Accepted.
6. HUD ship list rebuilds every 200 ms — allocation-heavy. Accepted.
7. `draw_indicators` (`content/ship/indicator.rs`) draws edge arrows for
   every ship, including ships well inside the viewport.
8. `.github/workflows/build-games.yml` references the pre-flatten `game/**`
   layout (`paths`, `working-directory`, cache paths, artifact path) so it
   never triggers and would fail outright; it also lacks `check`/`clippy`/
   `test` gates it implies. Fix: retarget paths to the workspace root.

# Known Defects (current implementation)

Factual defects in the current code, used as a work queue. Not a changelog.
Resolved items are removed; history lives in `openspec/changes/archive/`.

## Open

1. Parallax block (`PullingBeam` turret) and the Colaris Turret share
   `colaris.png` — no distinct art exists for the beam weapon.
2. `find_schematic` (`mission/state.rs`) silently falls back to the first
   available design when a mission references a missing `schematic_name` — a
   typo in a mission `.ron` spawns the wrong design with no error. The player
   slot documents this fallback (see mission.md); enemy slots have no such
   contract and should fail loudly or log.
3. `ShipBehavior` components are never removed — AI re-issues every think
   tick; behavior flips are invisible on screen. Accepted.
4. `handle_block_destroy` may write duplicate `DestroyBlock` events for
   multi-step accumulations — rare; duplicate drops for non-players. Accepted.
5. HUD ship list rebuilds every 200 ms — allocation-heavy. Accepted.
6. `draw_indicators` (`content/ship/indicator.rs`) draws edge arrows for
   every ship, including ships well inside the viewport.
7. `.github/workflows/build-games.yml` references the pre-flatten `game/**`
   layout (`paths`, `working-directory`, cache paths, artifact path) so it
   never triggers and would fail outright; it also lacks `check`/`clippy`/
   `test` gates it implies. Fix: retarget paths to the workspace root.

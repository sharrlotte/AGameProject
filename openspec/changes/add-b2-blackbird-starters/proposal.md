## Why

The built-in starter fleet offers only two combat-focused designs with near-identical roles, so a new player cannot experience distinct strategic roles from the shipped ships. Adding a durable long-range bomber and a fast recon scout gives the fleet real tactical variety while staying purely additive to existing data.

## What Changes

- Add two starter `ShipDesign` documents, `B2` and `Blackbird`, embedded and seeded alongside the existing starters.
- `B2` is a broad flying-wing bomber: two long-range `colaris` turrets, reinforced hull, minimal thrusters, and a stand-off `AiProfile` that keeps its distance and flees only at critical health. Its strengths are reach, survivability and sustained fire; its weaknesses are speed and turn rate.
- `Blackbird` is a long slender scout dart: one `laser_turret`, many small thrusters, one booster and a light power plant, with a mobile `AiProfile` that chases, kites and flees when damaged. Its strengths are speed and agility; its weaknesses are durability, firepower and endurance.
- Express each role through block composition and inline AI only: no `Role` component, stat modifiers, new block definitions or new art assets.
- Seed both designs on launch so they are available in the editor ship catalog and to missions.
- Set Quick Battle to player `Interceptor` against one `B2` and one `Blackbird`.
- No breaking changes; engine, save schema and serialization are untouched.

## Capabilities

### New Capabilities
- `starter-roles`: The two role-differentiated starter designs, their silhouettes, loadouts, AI profiles, and role-specific strengths and weaknesses.

### Modified Capabilities
- `world-save`: The fresh-install seeding requirement names only the interceptor and raider starters; it must also include the `B2` and `Blackbird` starter designs.

## Impact

- Config data: two new starter RON documents and an updated Quick Battle mission roster.
- Runtime: `save/design/lib.rs` gains two embedded constants and extended seed/test loops.
- Docs: spec-style updates to `docs/simulation.md` (starter composition table) and `docs/assets.md` (used config assets).
- No engine, schema, serialization, art, AI or block-definition changes; existing saves and designs remain compatible.
- Validation: both new designs must satisfy `validate_ship` (one core, connected cells, no overlap) and power feasibility, verified by tests and a dev run.

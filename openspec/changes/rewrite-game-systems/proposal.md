## Why

The game's core systems have correctness bugs that compound: destroyed block fragments remain in the collider grid, saves split ship data across files and restore ships at the origin, power exists as data but never affects gameplay, AI behaviors are unreachable, and continuous laser visuals point away from their own hitboxes. The same concepts are also represented by several overlapping structs and two serialization formats, so every fix risks another silent divergence. This change rewrites the affected systems around one source of truth per concept, one RON save format, and tests that pin the invariants.

## What Changes

- **Ship model:** `ShipGrid` becomes authoritative for placement and identity. Add `BlockPlacement`, `GridBlock`, `BlockRef`, and `ShipStats`; delete `BlockData`, `Block`, `BlockState`, `Blocks`, `ShipData`, `ShipState`, `ShipCalculatedStat`, `ShipCalculatedData`, `ShipFragment`, and `ComputedShipStats`. Destruction flows through one `ShipMutation` path that removes fragment cells, updates bounds/stats, and rebuilds the collider from live cells only.
- **Save/load:** **BREAKING** save format v2. RON only; remove `serde_json` and all JSON paths. Five fixed slots under `saves/slot_N.ron`; `WorldSave` embeds every `ShipSnapshot` (no `ship_ids`, no `ships/*.ron`). Full snapshot: transforms, velocities, health, ammo, AI, battle timer/outcome, drops, camera, RNG. Atomic write with `.tmp`/`.bak`; strict version rejection. Old JSON schematics and v1 saves are ignored and left untouched. Ship designs become versioned RON documents with starter designs copied on first run.
- **Power:** Add `PowerProducer` (core and new reactor) and `EnergyConsumer` (turrets/thrusters). Production is split equally among consumers; only consuming blocks get efficiency. Efficiency scales turret fire rate and thruster force/torque. Add a minimal production/demand/efficiency HUD.
- **Combat:** Turrets target the enemy core, not `blocks[0]`. Implement real `MostHP` (highest current ship HP) and `EnemyBullet` point defense against bullets and missiles (not lasers), with lead prediction. **Fix continuous laser and pulling-beam visuals by parenting them to the rotating barrel**, and make the projectile laser sprite offset axial instead of lateral. Guard empty barrels, apply pulling-beam pull once per ship per tick, and move damage text into camera range.
- **AI:** **BREAKING** replace 11 behavior marker components and `ShipBehavior` with one `AiBehavior` runtime enum plus a per-ship inline `AiProfile` of prioritized rules. Make `Regroup`, `Follow`, `Flee`, and `Kamikaze` reachable, fix the inverted flee direction, resolve real attack targets, and add an editor AI panel with presets.
- **Cargo:** Shared ship cargo pooled from all storage blocks. Capacity/usage/items live in one place; pickup is atomic and a full ship leaves drops in the world.
- **Scene lifecycle:** Replace manual `SceneEntity`/`clean_state_item` cleanup with `DespawnOnExit` scoping. Establish one documented schedule map, chain damage → death → ship mutation, and separate pilot control from camera panning.
- **Config:** Replace config/mission panics with `ConfigState::Error` and an error screen; validate blocks, power, turrets, shapes, teams, missions, starter designs, and AI profiles.
- **Code foundation:** Replace `lasso`/`Symbol`/`Symbols` with `String`. Add `RonDocument`, `Versioned<T>`, `CaptureSnapshot`/`RestoreSnapshot`, `feature_set!`, shared stat macros, and one type per concept. Delete dead `PowerConsumer`/`PowerStorage` and duplicate marker/DTO types.
- **Editor/UI:** Split the 1,419-line editor into feature modules; track the selected design by id; debounce autosave; add AI profile editing, save-slot UI, power/cargo HUD, and responsive panel widths.
- **Performance:** Replace 2,720 background sprites with a shader or tiled parallax layers, pool damage text, throttle radar/debug UI updates, and cache registry ordering.
- **Tests:** Add unit and integration coverage with every phase; each phase must compile, pass its tests, and remain playable.

## Capabilities

### New Capabilities

- `ship-grid`: canonical placement/grid model, `ShipStats`, mutation pipeline, fragment removal, core identity, bounds, and collider rebuild.
- `world-save`: RON-only document format, five embedded world slots, versioned and atomic IO, ship designs, and starter content.
- `power-system`: producers, consumers, equal-share efficiency, feature scaling, validation, and power HUD.
- `combat-targeting`: core targeting, `MostHP`, point defense, turret state, laser visual alignment, and combat edge cases.
- `ai-profiles`: per-ship inline AI profiles, prioritized rules, runtime behavior state, all behavior semantics, and editor authoring.
- `ship-cargo`: shared ship cargo capacity, item storage, and atomic drop collection.
- `scene-lifecycle`: state-scoped despawn, schedule contract, load transitions, and control modes.
- `config-validation`: recoverable config/mission errors, validation rules, and team constraints.
- `code-foundation`: string keys, RON document traits/macros, snapshot traits, feature/stat macros, and single-source-of-truth types.
- `editor-ui`: modular editor, design selection, debounced saving, AI panel, save-slot UI, and HUD surfaces.
- `performance`: background rendering, damage-text pooling, radar/debug update cadence, and registry caching.

### Modified Capabilities

- `collapsed-features`: block spawn and turret-child requirements change from `BlockData::spawn`/`PowerUsageComputed` to `BlockPlacement`/`BlockRef`/`GridBlock`, with weapon visuals parented to the barrel.
- `simulation-sets`: replace the `Input → Simulate → Apply → Cleanup` contract with the new schedule map, explicit damage/death/mutation chaining, and `DespawnOnExit` scene scoping.
- `symbol-registry`: **all requirements removed**; block and item keys are plain `String`s, not interned symbols.
- `ship-relative-save`: **all requirements removed**; world saves embed ships in one slot file.
- `ai-behavior-fsm`: **all requirements removed**; behavior is one runtime enum driven by per-ship profiles.
- `blocks-stats-cache`: **all requirements removed**; `ShipGrid` and `ShipStats` replace the separate stats cache.

## Impact

- Code: `content/ship/*`, `content/block/*`, `content/bullet/*`, `content/damage/*`, `content/death/*`, `save/*`, `mission/*`, `scene/*`, `config/*`, `system.rs`, `resource.rs`, `main.rs`, `symbol.rs` (deleted).
- Dependencies: remove `lasso` and `serde_json`; keep and standardize on `ron`.
- Assets: add `reactor` to `assets/config/base/block.ron`, starter RON designs, and AI defaults in ship designs.
- Data: new `designs/*.ron` and `saves/slot_1..5.ron`; old `schematics/*.json` and `world/` are not read and not migrated.
- Docs: update `docs/save.md`, `docs/combat.md`, `docs/ai.md`, `docs/architecture.md`, `docs/simulation.md`, `docs/config.md`, `docs/economy.md`, `docs/ui.md`, and `docs/defects.md` as phases land.
- Risk: the ship-grid and save rewrites touch most systems; mitigated by deleting parallel stores first, compiler-driven migration, and per-phase tests.

## Why

Several workspace crates had outgrown flat single-file modules—`combat/src/lib.rs` was 947 lines, `extraction/src/lib.rs` was 450 lines, and `presentation/src/` mixed `visuals.rs`, `hud.rs`, and `menus.rs`. While the immediate sub-feature split resolved the 500-line ceiling and established 1-word module files, scaling to a large-scale project (Phase 1–2: 7 zombie archetypes, procedural biomes/chunks, deployables, home base facilities, save/load, and future multiplayer) requires addressing fundamental structural bottlenecks:
1. `data` is currently a god-crate combining tags, game states, collision groups, cross-feature messages, upgrade resources, and generic RON loader infrastructure.
2. `presentation` directly depends on every simulation crate (`world`, `combat`, `enemies`, `extraction`), creating heavy coupling and cascade recompilation on every simulation change.
3. Cross-crate system ordering directly references foreign system functions (e.g., `world::player::apply_player_damage`) rather than explicit `SystemSet` contracts.
4. `core/src/main.rs` previously wired all plugins directly, while `core/assets/` lived inside an empty wrapper crate.
5. All `.ron` assets sit in a flat `config/` directory, which will not scale as biomes, room templates, loot tables, and weapons multiply.

## What Changes

### Stage 1: Nested Sub-Features & File Boundaries (Completed)
- Standardize the two-layer workspace structure keeping `crates/sim/` (headless simulation feature crates) and `crates/presentation/` (visual/client concern crate) strictly separated.
- Adopt a **nested sub-feature** convention inside `crates/sim/<feature>/src/` (`weapon/`, `projectile/`, `reload/`, `pickup/`, `zone/`, `flow/`, `upgrade/`, etc.), each containing standardized 1-word files (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`) as needed so every file stays under 500 lines.
- Enforce a **two-tier event model**: cross-feature Bevy messages stay in `crates/data/src/events.rs`, while internal events live in `<subfeature>/event.rs`.
- Reorganize `crates/presentation/src/` by visual concern (`camera/`, `input/`, `anim/`, `vfx/`, `audio/`, `scene/`, `ui/`), nesting feature sub-modules inside `scene/` and `ui/`.

### Stage 2: Large-Scale Architecture Enhancements (New Scope)
- **Split `data` God-Crate into Focused Crates:**
  - `common`: `GameState`, core `SystemSet`s (`SimUpdateSet`, `SimPostPhysicsSet`), collision `Group` constants, entity tags (`SurvivorTag`, `EnemyTag`, `ObstacleTag`, `RunEntity`), and shared enums (`EnemyKind`, `UpgradeKind`).
  - `config`: `RonConfig` trait, generic `RonAssetLoader<T>`, `DataPlugin`, `ConfigSyncSet`, `PendingConfigs`, `ConfigHotReloadedMessage`, and the `assert_ron_config_parses!` macro.
  - `message`: Cross-feature Bevy messages (`DamageMessage`, `EnemyKilledMessage`, `WeaponFiredMessage`, etc.) and shared progression/run resources (`RunStats`, `ScrapBank`, `LastRunOutcome`, `UpgradeLevels`, `UpgradeModifiers`).
- **Decouple `presentation` from Simulation Crates:**
  - Push shared queryable ECS components (`Survivor`, `Health`, `MovementStats`, `AimDirection`, `WeaponInventory`, `CombatStance`, `Enemy`, `ExtractionZone`, `ScrapPickup`) down into `common` (or a `sim-api` shared component layer), allowing `presentation` to render and bind animations purely by querying components without importing simulation crates.
  - Replace cross-crate foreign system function references with domain-level `SystemSet`s (e.g. `DamageResolutionSet`).
- **Root Workspace & Unified `game/` Crate:**
  - Move `Cargo.toml`, `Cargo.lock`, `.cargo/`, and `crates/` to the repository root so Cargo runs directly from root without nested `game/` wrappers.
  - Consolidate app assembly and binary entry into `game/` (`src/lib.rs` with `pub fn build_app() -> App`, `src/main.rs` calling `game::build_app().run()`, `tests/`, and `assets/`).
- **Domain-Grouped Assets in `game/assets/`:**
  - Group `game/assets/config/` into domain subfolders: `player/`, `combat/`, `enemies/`, `world/`, `extraction/`, `base/`.
  - Group `models/`, `audio/`, `texture/` along matching domain boundaries.
- **Pre-define Phase 2 Crate Skeletons:**
  - Establish crate boundaries for `crates/sim/automation/` (turrets, mines, traps, drones), `crates/sim/base/` (facilities, power, crafting, research), and `crates/persistence/` (profile save, expedition snapshots).
- **Feature-Flag Dev Dependencies:**
  - Move `bevy-inspector-egui` and dynamic linking behind explicit Cargo features (`dev`).
- **Add Integration Testing Infrastructure:**
  - Add headless integration tests (`game/tests/`) running simulation scenarios against `game::build_app()`.

## Capabilities

### New Capabilities

- `project-layout`:
  - Root-level workspace layering (`crates/{common,config,message,sim/*,persistence,presentation}`, `game`).
  - Nested sub-feature module conventions (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`).
  - Decoupled presentation querying shared ECS components rather than importing sim feature crates.
  - App assembly library (`game::build_app()`) enabling headless testing alongside the binary entry point in `game/`.
  - Domain-grouped asset directory layout (`game/assets/config/<domain>/...`).
  - Two-tier event ownership and cross-crate `SystemSet` contracts.
  - Strict file-size (<500 lines) and concise 1-word file naming invariants.

### Modified Capabilities

- `ron-config-loading`: Config file path constants updated to support domain subfolders (e.g., `config/combat/weapons.ron`) under `game/assets/`.

## Impact

- **Crate Graph**:
  - `Cargo.toml` and `crates/` moved to repository root.
  - `data` decomposes into `common`, `config`, and `message`.
  - `presentation` no longer depends on `world`, `combat`, `enemies`, `extraction`.
  - `game/` becomes the unified app crate (`default-members = ["game"]`) containing `src/`, `tests/`, and `assets/`.
- **Assets**:
  - Moved from `game/core/assets/` to `game/assets/`, with `.ron` config files in domain subfolders and `PATH` constants updated accordingly.
- **Documentation**:
  - `docs/07-technical-architecture-save-and-roadmap.md` updated with the multi-crate dependency graph and target layout.

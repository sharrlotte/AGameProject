## 1. Refactor `crates/sim/world` (Completed)

- [x] 1.1 Rename `game/crates/sim/world/src/input_intent.rs` to `intent.rs` and keep `ActionIntent` re-exported from `world::lib`
- [x] 1.2 Split `arena.rs` into `arena/{mod.rs, state.rs, system.rs}` and `player.rs` into `player/{mod.rs, state.rs, system.rs}`, preserving `ArenaPlugin`, `PlayerPlugin`, and all public component/system exports
- [x] 1.3 Run `cargo check -p world` to verify zero errors or warnings

## 2. Refactor `crates/sim/combat` (Completed)

- [x] 2.1 Extract weapon inventory state (`WeaponSlotState`, `WeaponInventory`) and weapon firing/swapping systems from `combat/src/lib.rs` into `weapon/{mod.rs, state.rs, system.rs}`
- [x] 2.2 Extract reload timer progression, completion/cancellation messages, and death cancellation into `reload/{mod.rs, system.rs}`
- [x] 2.3 Extract `Projectile` component, spawning, ticking, and Rapier collision handling into `projectile/{mod.rs, state.rs, system.rs}`
- [x] 2.4 Extract `CombatStance` and stance hold/update systems into `stance/{mod.rs, state.rs, system.rs}`
- [x] 2.5 Slim `combat/src/lib.rs` to module declarations, public re-exports, and `CombatPlugin` system-chain registration, then run `cargo check -p combat`

## 3. Refactor `crates/sim/enemies` (Completed)

- [x] 3.1 Split `enemies/src/lib.rs` into `spawn/{mod.rs, state.rs, system.rs}` (`EnemySpawner`, spawning), `ai/{mod.rs, state.rs, system.rs}` (`Enemy`, `FacingDirection`, chase/attack/freeze), and `damage/{mod.rs, system.rs}` (`apply_enemy_damage`)
- [x] 3.2 Update `enemies/src/lib.rs` to declare sub-feature modules, re-export public types, and wire `EnemiesPlugin` system chains
- [x] 3.3 Run `cargo test -p enemies` to confirm all enemy unit tests pass

## 4. Refactor `crates/sim/extraction` (Completed)

- [x] 4.1 Extract `ScrapPickup`, magnet pickup, and enemy-drop spawning into `pickup/{mod.rs, state.rs, system.rs}`
- [x] 4.2 Extract `ExtractionZone` and zone holdout progression into `zone/{mod.rs, state.rs, system.rs}`
- [x] 4.3 Extract run start/end/outcome flow into `flow/{mod.rs, system.rs}` and upgrade config/purchase systems into `upgrade/{mod.rs, config.rs, system.rs}`
- [x] 4.4 Update `extraction/src/lib.rs` to expose `ExtractionPlugin`, `RunFlowPlugin`, and `UpgradesPlugin`, then run `cargo check -p extraction`

## 5. Refactor `crates/presentation` (Completed)

- [x] 5.1 Move `camera.rs` into `camera/{mod.rs, system.rs}` (keeping `MainCamera` and `CameraPlugin` exports) and `input.rs` into `input/{mod.rs, system.rs}` (keeping `InputPlugin`)
- [x] 5.2 Split `visuals.rs` into `scene/{mod.rs, material.rs, arena.rs, player.rs, combat.rs, enemy.rs, extract.rs}` (and extract `HitFlash` into `vfx/{mod.rs, flash.rs}` if separated from `scene/`)
- [x] 5.3 Split `hud.rs` and `menus.rs` into `ui/{mod.rs, theme.rs, hud.rs, menu.rs}` sharing common bar/button helpers in `ui/theme.rs`
- [x] 5.4 Review `presentation/src/anim/` for consistency with concern-folder conventions (keeping `anim/mod.rs` under 500 lines and clean separation between `anim/` and `scene/player.rs`)
- [x] 5.5 Update `presentation/src/lib.rs` and `AllPresentationPlugins`, then run `cargo check -p presentation`

## 6. Documentation & Initial Verification (Completed)

- [x] 6.1 Update Section 2 of `docs/07-technical-architecture-save-and-roadmap.md` to document the refactored `sim/` sub-feature tree and `presentation/` concern layout
- [x] 6.2 Audit all `.rs` files in `game/crates/` to verify every file is under 500 lines, uses concise 1-word module/file names, and no `sim/*` crate depends on `presentation`
- [x] 6.3 Run `cargo check --workspace` and `cargo test --workspace` and verify all checks pass with zero warnings or errors

---

## 7. Foundation Layer Tripartite Split

- [x] 7.1 Create `crates/common`: Move `GameState`, `SimUpdateSet`, `SimPostPhysicsSet`, collision groups (`GROUP_*`), entity tags (`SurvivorTag`, `EnemyTag`, `ObstacleTag`, `RunEntity`), and shared enums (`EnemyKind`, `UpgradeKind`)
- [x] 7.2 Create `crates/config`: Move `RonConfig` trait, generic `RonAssetLoader<T>`, `DataPlugin`, `ConfigSyncSet`, `PendingConfigs`, `ConfigHotReloadedMessage`, and `assert_ron_config_parses!` macro
- [x] 7.3 Create `crates/message`: Move cross-feature Bevy `Message` definitions and shared resources (`RunStats`, `ScrapBank`, `LastRunOutcome`, `UpgradeLevels`, `UpgradeModifiers`)
- [x] 7.4 Update `game/Cargo.toml` workspace members and dependency specifications
- [x] 7.5 Verify with `cargo check -p common -p config -p message`

## 8. Presentation Decoupling & Shared Component Layer

- [x] 8.1 Move queryable components needed by presentation (`Survivor`, `Health`, `MovementStats`, `AimDirection`, `WeaponInventory`, `CombatStance`, `Enemy`, `ExtractionZone`, `ScrapPickup`) into `common` (or a shared component layer in `common`)
- [ ] 8.2 Remove dependencies on `world`, `combat`, `enemies`, and `extraction` from `crates/presentation/Cargo.toml`
- [ ] 8.3 Update `presentation` imports to query components from `common`
- [ ] 8.4 Run `cargo check -p presentation` to confirm presentation compiles without sim crate dependencies

## 9. Contract-Based Cross-Crate System Ordering

- [x] 9.1 Define shared system set contracts in `common` (e.g. `DamageResolutionSet`, `ProjectileCollisionSet`)
- [x] 9.2 Refactor `combat/src/lib.rs` system ordering to use `DamageResolutionSet` instead of direct function pointer `world::player::apply_player_damage`
- [x] 9.3 Run `cargo check -p combat -p world` to verify clean decoupled ordering

## 10. Root Workspace & Unified `game/` Crate

- [x] 10.1 Move `Cargo.toml`, `Cargo.lock`, `.cargo/`, and `crates/` to repository root (`e:/Codes/AGameProject/`)
- [x] 10.2 Consolidate app assembly and binary entry into `game/` (`src/lib.rs`, `src/main.rs`, `tests/`, and `assets/`)
- [x] 10.3 Run `cargo check -p game`

## 11. Domain-Grouped Asset Hierarchy

- [x] 11.1 Move `game/assets/config/*.ron` into domain subfolders (`player/`, `combat/`, `enemies/`, `world/`, `extraction/`, `base/`)
- [x] 11.2 Update all `RonConfig::PATH` constants and `assert_ron_config_parses!` to resolve `game/assets/`
- [x] 11.3 Run workspace config parse unit tests to verify all `.ron` configs resolve and parse cleanly

## 12. Dev Dependencies Feature-Flagging & Phase 2 Skeletons

- [x] 12.1 Make `bevy-inspector-egui` and dynamic linking optional under a `dev` cargo feature in `game/Cargo.toml`
- [x] 12.2 Create structural crate skeletons for `crates/sim/automation`, `crates/sim/base`, and `crates/persistence`
- [x] 12.3 Run `cargo check --workspace`

## 13. Workspace Integration Tests & Architecture Documentation

- [x] 13.1 Add headless integration tests under `game/tests/` running against `game::build_app()`
- [x] 13.2 Update `docs/07-technical-architecture-save-and-roadmap.md` and `AGENTS.md` with the root workspace and `game/` layout
- [x] 13.3 Run `cargo check --workspace` and `cargo test --workspace` to ensure zero warnings or failures

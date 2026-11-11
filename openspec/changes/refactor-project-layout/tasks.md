## 1. Refactor `crates/sim/world`

- [ ] 1.1 Rename `game/crates/sim/world/src/input_intent.rs` to `intent.rs` and keep `ActionIntent` re-exported from `world::lib`
- [ ] 1.2 Split `arena.rs` into `arena/{mod.rs, state.rs, system.rs}` and `player.rs` into `player/{mod.rs, state.rs, system.rs}`, preserving `ArenaPlugin`, `PlayerPlugin`, and all public component/system exports
- [ ] 1.3 Run `cargo check -p world` to verify zero errors or warnings

## 2. Refactor `crates/sim/combat`

- [ ] 2.1 Extract weapon inventory state (`WeaponSlotState`, `WeaponInventory`) and weapon firing/swapping systems from `combat/src/lib.rs` into `weapon/{mod.rs, state.rs, system.rs}`
- [ ] 2.2 Extract reload timer progression, completion/cancellation messages, and death cancellation into `reload/{mod.rs, system.rs}`
- [ ] 2.3 Extract `Projectile` component, spawning, ticking, and Rapier collision handling into `projectile/{mod.rs, state.rs, system.rs}`
- [ ] 2.4 Extract `CombatStance` and stance hold/update systems into `stance/{mod.rs, state.rs, system.rs}`
- [ ] 2.5 Slim `combat/src/lib.rs` to module declarations, public re-exports, and `CombatPlugin` system-chain registration, then run `cargo check -p combat`

## 3. Refactor `crates/sim/enemies`

- [ ] 3.1 Split `enemies/src/lib.rs` into `spawn/{mod.rs, state.rs, system.rs}` (`EnemySpawner`, spawning), `ai/{mod.rs, state.rs, system.rs}` (`Enemy`, `FacingDirection`, chase/attack/freeze), and `damage/{mod.rs, system.rs}` (`apply_enemy_damage`)
- [ ] 3.2 Update `enemies/src/lib.rs` to declare sub-feature modules, re-export public types, and wire `EnemiesPlugin` system chains
- [ ] 3.3 Run `cargo test -p enemies` to confirm all enemy unit tests pass

## 4. Refactor `crates/sim/extraction`

- [ ] 4.1 Extract `ScrapPickup`, magnet pickup, and enemy-drop spawning into `pickup/{mod.rs, state.rs, system.rs}`
- [ ] 4.2 Extract `ExtractionZone` and zone holdout progression into `zone/{mod.rs, state.rs, system.rs}`
- [ ] 4.3 Extract run start/end/outcome flow into `flow/{mod.rs, system.rs}` and upgrade config/purchase systems into `upgrade/{mod.rs, config.rs, system.rs}`
- [ ] 4.4 Update `extraction/src/lib.rs` to expose `ExtractionPlugin`, `RunFlowPlugin`, and `UpgradesPlugin`, then run `cargo check -p extraction`

## 5. Refactor `crates/presentation`

- [ ] 5.1 Move `camera.rs` into `camera/{mod.rs, system.rs}` (keeping `MainCamera` and `CameraPlugin` exports) and `input.rs` into `input/{mod.rs, system.rs}` (keeping `InputPlugin`)
- [ ] 5.2 Split `visuals.rs` into `scene/{mod.rs, material.rs, arena.rs, player.rs, combat.rs, enemy.rs, extract.rs}` (and extract `HitFlash` into `vfx/{mod.rs, flash.rs}` if separated from `scene/`)
- [ ] 5.3 Split `hud.rs` and `menus.rs` into `ui/{mod.rs, theme.rs, hud.rs, menu.rs}` sharing common bar/button helpers in `ui/theme.rs`
- [ ] 5.4 Review `presentation/src/anim/` for consistency with concern-folder conventions (keeping `anim/mod.rs` under 500 lines and clean separation between `anim/` and `scene/player.rs`)
- [ ] 5.5 Update `presentation/src/lib.rs` and `AllPresentationPlugins`, then run `cargo check -p presentation`

## 6. Documentation & Full Workspace Verification

- [ ] 6.1 Update Section 2 of `docs/07-technical-architecture-save-and-roadmap.md` to document the refactored `sim/` sub-feature tree and `presentation/` concern layout
- [ ] 6.2 Audit all `.rs` files in `game/crates/` to verify every file is under 500 lines, uses concise 1-word module/file names, and no `sim/*` crate depends on `presentation`
- [ ] 6.3 Run `cargo check --workspace` and `cargo test --workspace` and verify all checks pass with zero warnings or errors

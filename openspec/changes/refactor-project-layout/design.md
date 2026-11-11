## Context

The workspace currently uses 7 crates (`data`, `world`, `combat`, `enemies`, `extraction`, `presentation`, `core`) in `game/Cargo.toml`:
- `crates/data/` (4 files, 39–238 lines each) holds `GameState`, shared tags, `events.rs`, and the RON config loader.
- `crates/sim/combat/src/lib.rs` has grown to 947 lines in a single file (well above the 500-line ceiling in `AGENTS.md`), and `crates/sim/extraction/src/lib.rs` is 450 lines containing three plugins (`ExtractionPlugin`, `RunFlowPlugin`, `UpgradesPlugin`).
- `crates/sim/enemies/src/lib.rs` is 341 lines and `crates/sim/world/src/` uses flat feature files (`arena.rs`, `player.rs`, `input_intent.rs`, `config.rs`).
- `crates/presentation/src/` mixes flat files (`visuals.rs` at 357 lines, `hud.rs` at 417 lines, `menus.rs` at 367 lines, `camera.rs`, `input.rs`) with the newer `anim/` folder (9 concise 1-word files, 52–369 lines).

The codebase is about to expand into Phase 1–2 features (7 zombie archetypes, procedural biomes/chunks, deployables/drones, home base facilities, save/load). Refactoring the layout now establishes a repeatable module structure without changing runtime gameplay.

## Goals / Non-Goals

**Goals:**
- Keep the strict two-layer separation between `crates/sim/*` (headless simulation) and `crates/presentation` (client visuals, audio, UI, input, camera, animation).
- Keep feature crates in `crates/sim/` independent for fast incremental/parallel Cargo compilation.
- Split oversized simulation crates and flat presentation files into nested sub-features and concern folders using concise 1-word file names under 500 lines each.
- Enforce two-tier event ownership (`data/src/events.rs` for cross-feature messages, `<subfeature>/event.rs` for local messages).
- Keep `docs/07-technical-architecture-save-and-roadmap.md` in sync with the implemented directory layout.

**Non-Goals:**
- Changing gameplay mechanics, physics, enemy behavior, UI appearance, or `.ron` schema/loading behavior (`ron-config-loading`).
- Merging `sim/*` crates into one monolithic crate or splitting `presentation` into multiple Cargo crates at this stage.
- Implementing Phase 1–2 features (`deploy`, `base`, `save`) prematurely; only prepare the layout conventions and folder boundaries they will follow.
- Rewriting the layered animation design in `player-animation-layers` (only align module structure/file boundaries where needed).

## Decisions

### D1. Two-Layer Cargo Boundary (`crates/sim/` vs `crates/presentation/`)
- **Choice:** Retain `crates/sim/<feature>/` feature crates and a single `crates/presentation/` leaf crate, matching `docs/07` naming.
- **Rationale:** Simulation crates stay headless-capable (future co-op/server and unit tests require zero rendering/UI dependencies), while editing presentation/UI never triggers downstream recompiles in simulation crates.
- **Alternatives considered:**
  - *Single `logic` crate + single `visual` crate*: fewer `Cargo.toml` files, but any edit in `combat` would recompile `enemies` and `extraction`.
  - *UI files inside feature crates behind `#[cfg(feature = "client")]`*: creates feature-flag boilerplate, couples headless sim to UI widgets, and triggers sim recompiles on UI tweaks.

### D2. Nested Sub-Features with Standardized 1-Word Files in `crates/sim/`
- **Choice:** Each `sim/<feature>/src/` crate splits into nested sub-feature folders where each sub-feature uses standardized 1-word file names (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`, plus `tests.rs` where needed). Only the files a sub-feature actually uses are created (no empty stub files). Shared crate-level `.ron` config stays in `config.rs` at the crate root when one `.ron` file feeds multiple sub-features (e.g., `combat`, `enemies`, `extraction`), with sub-feature `config.rs` re-exporting or holding sub-structs if split later.
- **Mapping for existing sim crates:**
  - `sim/combat/src/`: `lib.rs` (plugin wiring + `CombatPlugin`), `config.rs` (`WeaponsConfig`), `weapon/` (`state.rs`, `system.rs`), `reload/` (`system.rs`), `projectile/` (`state.rs`, `system.rs`), `stance/` (`state.rs`, `system.rs`), `tests.rs` (if unit tests are added/moved).
  - `sim/extraction/src/`: `lib.rs` (re-exports `ExtractionPlugin`, `RunFlowPlugin`, `UpgradesPlugin`), `config.rs`, `pickup/` (`state.rs`, `system.rs`), `zone/` (`state.rs`, `system.rs`), `flow/` (`system.rs`), `upgrade/` (`config.rs`, `system.rs`).
  - `sim/enemies/src/`: `lib.rs`, `config.rs`, `spawn/` (`state.rs`, `system.rs`), `ai/` (`state.rs`, `system.rs`), `damage/` (`system.rs`), `tests.rs`.
  - `sim/world/src/`: `lib.rs`, `config.rs`, `intent.rs` (renamed from `input_intent.rs` for 1-word naming while re-exporting `ActionIntent`), `arena/` (`state.rs`, `system.rs`), `player/` (`state.rs`, `system.rs`).
- **Rationale:** Splitting by domain sub-feature prevents `system.rs` from exceeding 500 lines and makes every feature predictable to navigate.
- **Alternatives considered:**
  - *Single `system.rs` at crate root*: would immediately create a ~900-line `combat/src/system.rs`, violating the 500-line limit.
  - *Mandatory empty `event.rs`/`config.rs` in every folder*: creates noise files with zero types.

### D3. Two-Tier Event Placement
- **Choice:** Cross-feature typed Bevy `Message`s remain in `crates/data/src/events.rs`. Events produced and consumed solely within a single feature or sub-feature live in `<subfeature>/event.rs` and are registered by that feature's plugin.
- **Rationale:** Prevents circular Cargo dependencies between `combat`, `enemies`, and `extraction` while allowing sub-features to decouple internally without polluting `data`.

### D4. Concern-First `crates/presentation/` with Feature Sub-Modules in `scene/` and `ui/`
- **Choice:** Keep `presentation` as one crate with top-level directories organized by visual concern:
  - `camera/` (`mod.rs`, `system.rs` — migrated from `camera.rs`)
  - `input/` (`mod.rs`, `system.rs` — migrated from `input.rs`)
  - `anim/` (existing `PlayerAnimPlugin` submodules; ensure `anim/mod.rs` remains `<500` lines and only owns animation/rig binding)
  - `vfx/` (module shell or extracted hit-flash/particle helpers ready for `bevy_hanabi`)
  - `audio/` (module shell ready for Phase 1–3 spatial audio)
  - `scene/` (migrated from `visuals.rs`: `mod.rs`, `material.rs`, `arena.rs`, `player.rs`, `combat.rs`, `enemy.rs`, `extract.rs`)
  - `ui/` (migrated from `hud.rs` + `menus.rs`: `mod.rs`, `theme.rs`, `hud.rs` or `hud/`, `menu.rs` or `menu/`)
- **Rationale:** Preserves shared visual pipelines (`camera`, `input`, `anim`, `vfx`, `audio`, `ui/theme`) while preventing `visuals.rs`, `hud.rs`, and `menus.rs` from growing past 500 lines as biomes, enemies, deployables, and base UI screens are added.
- **Alternatives considered:**
  - *Purely flat concern folders (`ui.rs`, `scene.rs`)*: would merge `hud.rs` (417) + `menus.rs` (367) into ~784 lines immediately.
  - *Splitting `presentation` into 6 Cargo crates now*: premature Cargo overhead before compile times warrant it; concern folders can be promoted to crates later with minimal import changes.

### D5. Public Crate Re-Exports & System Ordering Preservation
- **Choice:** Keep each crate's root `lib.rs` re-exporting its public plugins, components, configs, and system sets (`CombatPlugin`, `EnemiesPlugin`, `ExtractionPlugin`, `RunFlowPlugin`, `UpgradesPlugin`, `ArenaPlugin`, `PlayerPlugin`, `AllPresentationPlugins`, etc.) so `game/core/src/main.rs` and cross-crate imports remain clean and system `.chain()` / `SimUpdateSet` / `SimPostPhysicsSet` ordering stays identical.

## Risks / Trade-offs

- **[Risk] Overlapping edits with active `player-animation-layers` change** → Keep `presentation/src/anim/` public exports (`PlayerAnimPlugin`, `PlayerVisualRoot`, `PlayerFacing`, `PlayerRig`, `AnimCatalog`, `PlayerIkSet`) and `sim/world` / `sim/combat` public symbols (`ActionIntent`, `CombatStance`, reload/swap messages) stable so in-flight animation work merges cleanly.
- **[Risk] Broken system ordering during extraction from monolithic `lib.rs`** → Preserve exact `.chain()`, `.in_set(SimUpdateSet)`, `.in_set(SimPostPhysicsSet)`, `.after(...)`, and `.before(...)` constraints when moving systems into sub-feature `system.rs` files, and verify with `cargo check` and `cargo test` after each crate migration.
- **[Risk] Empty placeholder folders (`vfx/`, `audio/`) triggering dead-code warnings** → Only expose clean module/plugin hooks or add `vfx/` / `audio/` when extracting actual existing logic (e.g. `HitFlash` in `vfx` or keeping `audio/` documented in `docs/07` until audio systems are added) so there is zero unused code or lint suppression.

## Migration Plan

1. Refactor `crates/sim/world` (lightweight alignment + `intent.rs`).
2. Refactor `crates/sim/combat` (split 947-line `lib.rs` into `weapon/`, `projectile/`, `reload/`, `stance/`).
3. Refactor `crates/sim/enemies` (`spawn/`, `ai/`, `damage/`, `tests.rs`).
4. Refactor `crates/sim/extraction` (`pickup/`, `zone/`, `flow/`, `upgrade/`).
5. Refactor `crates/presentation` (`camera/`, `input/`, `scene/`, `ui/`, and `anim/` consistency check).
6. Update `docs/07-technical-architecture-save-and-roadmap.md` and run `cargo check --workspace` and `cargo test --workspace` to verify zero regressions.

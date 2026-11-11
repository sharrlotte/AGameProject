## ADDED Requirements

### Requirement: Two-Layer Crate Separation
The workspace SHALL enforce a strict separation between simulation crates under `game/crates/sim/` and the client presentation crate under `game/crates/presentation/`. Simulation crates MUST NOT depend on `presentation` or any rendering, windowing, audio, or UI crate.

#### Scenario: Simulation compiles independently of presentation
- **WHEN** inspecting `Cargo.toml` dependencies and `use` imports across `game/crates/sim/*`
- **THEN** no simulation crate depends on `presentation` or Bevy UI/rendering modules, and `cargo test --workspace` compiles and runs simulation tests headlessly without presentation dependencies

### Requirement: Nested Sub-Feature Structure with Standardized 1-Word Files
Each simulation feature crate under `game/crates/sim/<feature>/src/` SHALL organize domain logic into nested sub-feature directories (such as `weapon/`, `projectile/`, `reload/`, `pickup/`, `zone/`, `flow/`, `upgrade/`, `spawn/`, `ai/`, `damage/`, `arena/`, `player/`), using concise 1-word file names (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`, `tests.rs`) for the roles present in that sub-feature. Every Rust source file in the refactored layout MUST remain under 500 lines.

#### Scenario: Oversized simulation crates are split into sub-features under 500 lines
- **WHEN** `sim/combat` and `sim/extraction` are refactored into nested sub-features
- **THEN** `sim/combat/src/lib.rs` and `sim/extraction/src/lib.rs` contain only module declarations, re-exports, and plugin registration/wiring, and every `.rs` file in `game/crates/` is under 500 lines

#### Scenario: Sub-feature files use 1-word names
- **WHEN** a sub-feature directory is created under `game/crates/sim/<feature>/src/<subfeature>/`
- **THEN** its source files use standardized 1-word names (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`, `tests.rs`) and omit files for roles the sub-feature does not use

### Requirement: Two-Tier Event Ownership
Cross-feature typed Bevy messages SHALL reside in `game/crates/data/src/events.rs`, while feature-internal or sub-feature-internal messages SHALL reside in the owning module's `event.rs`. Simulation crates MUST NOT introduce circular Cargo dependencies.

#### Scenario: Cross-feature vs internal message placement
- **WHEN** a message is consumed across multiple simulation crates or by `presentation`
- **THEN** it is defined in `data/src/events.rs`, whereas messages confined to a single sub-feature live in `<subfeature>/event.rs`

### Requirement: Concern-Based Presentation Layout with Feature Sub-Modules
The `game/crates/presentation/src/` crate SHALL organize top-level directories by visual concern (`camera/`, `input/`, `anim/`, `scene/`, `ui/`, and `vfx/` when visual effects are extracted). Concerns that scale with gameplay features (`scene/` and `ui/`) MUST subdivide internally by feature or screen (`scene/{arena,player,combat,enemy,extract}.rs`, `ui/{theme,hud,menu}.rs` or sub-folders) so each file stays under 500 lines.

#### Scenario: Presentation visuals, HUD, and menus are decomposed by concern and feature
- **WHEN** `presentation/src/visuals.rs`, `hud.rs`, `menus.rs`, `camera.rs`, and `input.rs` are refactored
- **THEN** 3D mesh/material spawning lives under `scene/`, HUD and menu screens live under `ui/` sharing common builders in `ui/theme.rs`, camera and input live in `camera/` and `input/`, and `AllPresentationPlugins` continues to register all presentation plugins cleanly

#### Scenario: Animation concern module alignment
- **WHEN** `presentation/src/anim/` is reviewed alongside `scene/`
- **THEN** `anim/` retains concise 1-word module names under 500 lines, keeps animation/rig logic decoupled from unrelated scene spawning, and preserves its public plugin and component exports

### Requirement: Preserved Schedule Ordering and Runtime Behavior
Refactoring modules and files into the new project layout SHALL preserve all existing plugin registrations, `SimUpdateSet` / `SimPostPhysicsSet` / `PhysicsSet` / `TransformSystems` system ordering, RON hot-reload behavior, and runtime gameplay behavior.

#### Scenario: Workspace builds and passes all checks after refactor
- **WHEN** `cargo check --workspace` and `cargo test --workspace` are executed after the layout migration
- **THEN** all crates compile with zero warnings/errors and all existing tests pass without behavior regressions

### Requirement: Architecture Documentation Synchronization
The technical architecture document `docs/07-technical-architecture-save-and-roadmap.md` SHALL be updated to reflect the refactored directory tree, sub-feature conventions, and presentation concern structure.

#### Scenario: Documentation matches implemented layout
- **WHEN** inspecting Section 2 of `docs/07-technical-architecture-save-and-roadmap.md` after the refactor
- **THEN** the documented crate and module tree matches the actual directories under `game/crates/`

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
Cross-feature typed Bevy messages SHALL reside in the shared message layer (`game/crates/message/`), while feature-internal or sub-feature-internal messages SHALL reside in the owning module's `event.rs`. Simulation crates MUST NOT introduce circular Cargo dependencies.

#### Scenario: Cross-feature vs internal message placement
- **WHEN** a message is consumed across multiple simulation crates or by `presentation`
- **THEN** it is defined in `message/src/lib.rs` (or `events.rs`), whereas messages confined to a single sub-feature live in `<subfeature>/event.rs`

### Requirement: Tripartite Foundation Layer
The foundation infrastructure SHALL be split into three focused crates: `crates/common/` (leaf state, tags, system sets, shared queryable components), `crates/config/` (generic RON asset loader, trait, and DataPlugin), and `crates/message/` (cross-feature messages and shared run/upgrade resources).

#### Scenario: Foundation crates do not pull in unnecessary dependencies
- **WHEN** a simulation crate only requires shared components or system sets
- **THEN** it depends on `common` without pulling in message or asset loader churn

### Requirement: Presentation Decoupling via Shared Components
The `crates/presentation/` crate SHALL query shared ECS components defined in `common` (or a shared component layer) and SHALL NOT depend directly on `sim/world`, `sim/combat`, `sim/enemies`, or `sim/extraction`.

#### Scenario: Presentation compiles without simulation crate dependencies
- **WHEN** inspecting `crates/presentation/Cargo.toml`
- **THEN** `presentation` does not list `world`, `combat`, `enemies`, or `extraction` as dependencies, yet all camera, input, animation, scene, UI, and VFX systems query the necessary game state and components via `common`

### Requirement: Contract-Based Cross-Crate System Ordering
All system schedule ordering across crate boundaries SHALL use explicit named `SystemSet`s defined in `common`. Systems in one crate MUST NOT reference private or internal system functions in another crate with `.after()` or `.before()`.

#### Scenario: Ordering across crates uses SystemSets
- **WHEN** a system in `combat` must run after player damage resolution in `world`
- **THEN** both systems order themselves against a shared `SystemSet` contract (e.g. `DamageResolutionSet`) without direct function imports

### Requirement: App Assembly Library and Headless Testability
The workspace SHALL provide a `crates/game` library crate that exposes `pub fn build_app() -> App` wiring all simulation, presentation, and data plugins. The binary crate `core/src/main.rs` SHALL serve as a thin entry point that invokes `game::build_app().run()`.

#### Scenario: Headless integration tests run without UI dependencies
- **WHEN** integration tests under `game/tests/` instantiate the game loop
- **THEN** they build the application via `game::build_app()` (or a headless variant) and advance frames to test end-to-end combat, progression, and extraction flows

### Requirement: Domain-Grouped Asset Hierarchy
Runtime assets in `core/assets/` SHALL be organized into domain directories for configuration (`config/<domain>/...`), 3D models (`models/<domain>/...`), audio, and textures.

#### Scenario: RON configuration files reside in domain subfolders
- **WHEN** reading `.ron` configuration files
- **THEN** they are located in `config/player/`, `config/combat/`, `config/enemies/`, `config/world/`, `config/extraction/`, and `config/base/`, and `RonConfig::PATH` constants accurately reflect these paths

### Requirement: Concern-Based Presentation Layout with Feature Sub-Modules
The `game/crates/presentation/src/` crate SHALL organize top-level directories by visual concern (`camera/`, `input/`, `anim/`, `scene/`, `ui/`, and `vfx/`). Concerns that scale with gameplay features (`scene/` and `ui/`) MUST subdivide internally by feature or screen so each file stays under 500 lines.

#### Scenario: Presentation visuals, HUD, and menus are decomposed by concern and feature
- **WHEN** `presentation/src/visuals.rs`, `hud.rs`, `menus.rs`, `camera.rs`, and `input.rs` are refactored
- **THEN** 3D mesh/material spawning lives under `scene/`, HUD and menu screens live under `ui/` sharing common builders in `ui/theme.rs`, camera and input live in `camera/` and `input/`, and `AllPresentationPlugins` continues to register all presentation plugins cleanly

### Requirement: Architecture Documentation Synchronization
The technical architecture document `docs/07-technical-architecture-save-and-roadmap.md` SHALL be updated to reflect the refactored directory tree, sub-feature conventions, and presentation concern structure.

#### Scenario: Documentation matches implemented layout
- **WHEN** inspecting Section 2 of `docs/07-technical-architecture-save-and-roadmap.md` after the refactor
- **THEN** the documented crate and module tree matches the actual directories under `game/crates/`

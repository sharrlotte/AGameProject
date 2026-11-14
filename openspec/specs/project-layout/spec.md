# project-layout Specification

## Purpose
Two-crate workspace (`engine` library + `game` binary) with nested sub-feature modules, intra-crate module decoupling, SystemSet-based ordering, a unified `game/` app crate, domain-grouped assets, synchronized architecture documentation, and enforced file-size/test-placement/lint rules.

## Requirements

### Requirement: Two-Crate Workspace Separation
The workspace SHALL contain exactly two crates: `engine` (library: simulation, presentation, config, messages, base) and `game` (binary + assets + integration tests). `game` depends on `engine`; `engine` MUST NOT depend on `game`. Simulation modules inside `engine` MUST NOT depend on rendering, windowing, audio, or UI modules.

#### Scenario: Engine compiles independently of game binary concerns
- **WHEN** inspecting `engine/Cargo.toml` dependencies
- **THEN** `engine` depends only on third-party crates and exposes library targets with no dependency on `game`

#### Scenario: Simulation modules do not import presentation
- **WHEN** inspecting `use` statements under `engine/src/world`, `engine/src/combat`, `engine/src/enemies`, `engine/src/extraction`, `engine/src/hazard`
- **THEN** none import from `engine::presentation` or Bevy rendering/UI modules

### Requirement: Engine Foundation Modules
The foundation infrastructure SHALL live as modules of `engine`: `engine/src/common/` (leaf state, tags, system sets, shared components), `engine/src/config/` (RON config loading, `RonConfig` trait, `ConfigPlugin`), and `engine/src/message/` (cross-feature messages and shared run/upgrade resources). These modules MUST remain dependency-light relative to feature modules.

#### Scenario: Foundation modules are the leaf layer
- **WHEN** a feature module needs shared components or system sets
- **THEN** it depends on `engine::common` without pulling in `engine::presentation` or feature modules

### Requirement: Intra-Crate Module Decoupling
The `engine/src/presentation/` module SHALL query shared ECS components defined in `engine::common` and MUST NOT depend directly on `engine::world`, `engine::combat`, `engine::enemies`, or `engine::extraction` feature modules.

#### Scenario: Presentation stays decoupled from feature modules
- **WHEN** inspecting imports under `engine/src/presentation/`
- **THEN** it references only `engine::common`, `engine::config`, `engine::message`, and third-party crates

### Requirement: Intra-Crate System Ordering via SystemSets
All system schedule ordering SHALL use explicit named `SystemSet`s defined in `engine::common`. Modules MUST NOT reference private system functions in other modules with `.after()` or `.before()`.

#### Scenario: Ordering uses shared SystemSets
- **WHEN** a system in `engine::combat` must run after player damage resolution in `engine::world`
- **THEN** both order against a shared `SystemSet` (e.g. `DamageResolutionSet`) without direct function imports

### Requirement: Nested Sub-Feature Structure with Standardized 1-Word Files
Each feature module under `engine/src/<feature>/` SHALL organize domain logic into nested sub-feature directories (such as `weapon/`, `projectile/`, `reload/`, `pickup/`, `zone/`, `flow/`, `upgrade/`, `spawn/`, `ai/`, `damage/`, `arena/`, `player/`), using concise 1-word file names (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`, `tests.rs`) for the roles present in that sub-feature. Every Rust source file MUST remain under 500 lines. Tests MAY be co-located inline with `#[cfg(test)]` or placed in a separate `tests.rs` file, at the author's discretion.

#### Scenario: Oversized modules are split into sub-features under 500 lines
- **WHEN** a module exceeds 500 lines
- **THEN** it is split into sub-feature directories so every `.rs` file is under 500 lines

#### Scenario: Sub-feature files use 1-word names
- **WHEN** a sub-feature directory is created under `engine/src/<feature>/<subfeature>/`
- **THEN** its source files use standardized 1-word names (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`, `tests.rs`) and omit files for roles the sub-feature does not use

### Requirement: Two-Tier Event Ownership
Cross-feature typed Bevy messages SHALL reside in the shared message layer (`engine/src/message/`), while feature-internal or sub-feature-internal messages SHALL reside in the owning module's `event.rs`. Modules MUST NOT introduce circular dependencies.

#### Scenario: Cross-feature vs internal message placement
- **WHEN** a message is consumed across multiple feature modules or by `presentation`
- **THEN** it is defined in `engine/src/message/`, whereas messages confined to a single sub-feature live in `<subfeature>/event.rs`

### Requirement: Unified App Assembly & Binary Crate
The workspace SHALL sit at the repository root (`Cargo.toml`) and provide an `engine` library crate plus a `game` crate containing both a library target (`pub fn build_app() -> App` wiring all simulation, presentation, and data plugins in `game/src/lib.rs`) and a thin binary entry point (`game/src/main.rs`) invoking `game::build_app().run()`.

#### Scenario: Headless integration tests run without UI dependencies
- **WHEN** integration tests under `game/tests/` instantiate the game loop
- **THEN** they build the application via `game::build_app()` and verify plugin/state initialization or advance frames headlessly

### Requirement: Domain-Grouped Asset Hierarchy in `game/assets/`
Runtime assets SHALL reside at `game/assets/` and be organized into domain directories for configuration (`config/<domain>/...`), 3D models (`models/<domain>/...`), audio, and textures.

#### Scenario: RON configuration files reside in domain subfolders
- **WHEN** reading `.ron` configuration files
- **THEN** they are located in `config/player/`, `config/combat/`, `config/enemies/`, `config/world/`, `config/extraction/`, and `config/base/`, and `RonConfig::PATH` constants accurately reflect these paths

### Requirement: Concern-Based Presentation Layout with Feature Sub-Modules
The `engine/src/presentation/` module SHALL organize top-level directories by visual concern (`camera/`, `input/`, `anim/`, `scene/`, `ui/`, and `vfx/`). Concerns that scale with gameplay features (`scene/` and `ui/`) MUST subdivide internally by feature or screen so each file stays under 500 lines.

#### Scenario: Presentation visuals, HUD, and menus are decomposed by concern and feature
- **WHEN** presentation files are refactored
- **THEN** 3D mesh/material spawning lives under `scene/`, HUD and menu screens live under `ui/` sharing common builders in `ui/theme.rs`, camera and input live in `camera/` and `input/`, and `AllPresentationPlugins` continues to register all presentation plugins cleanly

### Requirement: Architecture Documentation Synchronization
The technical architecture document `docs/07-technical-architecture-save-and-roadmap.md` SHALL be updated to reflect the two-crate tree, sub-feature conventions, and presentation concern structure, including accurate module paths under `engine/src/`.

#### Scenario: Documentation matches implemented layout
- **WHEN** inspecting Section 2 of `docs/07-technical-architecture-save-and-roadmap.md` after the refactor
- **THEN** the documented crate and module tree matches the actual directories under `engine/src/` and `game/`

### Requirement: Modular weapon configuration sub-modules
The unified weapon configuration SHALL be split into focused sub-modules under `engine/src/config/weapons/` (`grip.rs`, `sockets.rs`, `model.rs`, `presentation.rs`, `stats.rs`, `roster.rs`), with `mod.rs` re-exporting all types, ensuring every source file remains under 500 lines.

#### Scenario: All weapon config files are under 500 lines
- **WHEN** inspecting lines of code in `engine/src/config/weapons/*`
- **THEN** each file is strictly under 500 lines and adheres to single responsibility

### Requirement: Home base progression module placement
Meta-progression upgrades, upgrade cost formulas, modifiers, and scrap banking logic SHALL reside in `engine::base` (`BasePlugin`), matching the project roadmap.

#### Scenario: Upgrades registered through BasePlugin
- **WHEN** the application starts
- **THEN** `BasePlugin` registers upgrade resources, configs, and purchase systems instead of `ExtractionPlugin`

### Requirement: Strict dev tooling feature gating
Development and remote debugging plugins — specifically `bevy::remote::RemotePlugin` and `BrpExtrasPlugin` — SHALL be compiled and registered only when the `dev` cargo feature is active, and production builds (`--no-default-features`) SHALL compile without opening remote network ports.

#### Scenario: Release build compiles without dev plugins
- **WHEN** building the game without the `dev` feature (`cargo build --no-default-features`)
- **THEN** the compilation succeeds and no BRP or inspector plugins are instantiated

### Requirement: Standards Enforcement (file size, imports, lint)
The codebase SHALL enforce the AGENTS.md rules: every Rust source file under 500 lines; top-level `use` imports instead of inline fully-qualified `crate::` paths in production code; and a clean clippy baseline gated by `clippy.toml`. Tests may be co-located inline or in separate files.

#### Scenario: Clippy baseline is clean
- **WHEN** running `cargo clippy -p engine`
- **THEN** it completes with zero warnings under the configured `clippy.toml` rules
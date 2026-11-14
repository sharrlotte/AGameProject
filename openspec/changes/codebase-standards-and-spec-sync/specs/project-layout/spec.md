## REMOVED Requirements

### Requirement: Two-Layer Crate Separation
**Reason**: The workspace collapsed to two crates (`engine` library + `game` binary) on 2026-11-13. Simulation and presentation now live as modules inside `engine`, separated by module boundaries rather than crate dependencies.
**Migration**: Use the `Two-Crate Workspace Separation` requirement added below; module-level decoupling rules replace crate-level ones.

### Requirement: Tripartite Foundation Layer
**Reason**: `common`, `config`, and `message` are now modules of `engine` (`engine/src/common`, `engine/src/config`, `engine/src/message`), not separate crates. The foundation is a single crate with internal modules.
**Migration**: Use the `Engine Foundation Modules` requirement added below.

### Requirement: Presentation Decoupling via Shared Components
**Reason**: Presentation is a module (`engine/src/presentation`) inside `engine`, not a separate crate, so it cannot have its own `Cargo.toml` dependency list. Decoupling is enforced at the module level.
**Migration**: Use the `Intra-Crate Module Decoupling` requirement added below.

### Requirement: Contract-Based Cross-Crate System Ordering
**Reason**: System ordering is now intra-crate; there is no cross-crate boundary. The `SystemSet` contract pattern is retained but scoped within `engine`.
**Migration**: Use the `Intra-Crate System Ordering via SystemSets` requirement added below.

## ADDED Requirements

### Requirement: Two-Crate Workspace Separation
The workspace SHALL contain exactly two crates: `engine` (library: simulation, presentation, config, messages, base) and `game` (binary + assets + integration tests). `game` depends on `engine`; `engine` MUST NOT depend on `game`. Simulation modules inside `engine` MUST NOT depend on rendering, windowing, audio, or UI modules.

#### Scenario: Engine compiles independently of game binary concerns
- **WHEN** inspecting `engine/Cargo.toml` dependencies
- **THEN** `engine` depends only on third-party crates (bevy, rapier, ron, serde, etc.) and exposes `build_app`-independent library targets, with no dependency on `game`

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

## MODIFIED Requirements

### Requirement: Unified App Assembly & Binary Crate
The workspace SHALL sit at the repository root (`Cargo.toml`) and provide an `engine` library crate plus a `game` crate containing both a library target (`pub fn build_app() -> App` wiring all simulation, presentation, and data plugins in `game/src/lib.rs`) and a thin binary entry point (`game/src/main.rs`) invoking `game::build_app().run()`.

#### Scenario: Headless integration tests run without UI dependencies
- **WHEN** integration tests under `game/tests/` instantiate the game loop
- **THEN** they build the application via `game::build_app()` and verify plugin/state initialization or advance frames headlessly

### Requirement: Architecture Documentation Synchronization
The technical architecture document `docs/07-technical-architecture-save-and-roadmap.md` SHALL be updated to reflect the two-crate tree, sub-feature conventions, and presentation concern structure, including accurate module paths under `engine/src/`.

#### Scenario: Documentation matches implemented layout
- **WHEN** inspecting Section 2 of `docs/07-technical-architecture-save-and-roadmap.md` after the refactor
- **THEN** the documented crate and module tree matches the actual directories under `engine/src/` and `game/`

### Requirement: Nested Sub-Feature Structure with Standardized 1-Word Files
Each feature module under `engine/src/<feature>/` SHALL organize domain logic into nested sub-feature directories (such as `weapon/`, `projectile/`, `reload/`, `pickup/`, `zone/`, `flow/`, `upgrade/`, `spawn/`, `ai/`, `damage/`, `arena/`, `player/`), using concise 1-word file names (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`, `tests.rs`) for the roles present in that sub-feature. Every Rust source file MUST remain under 500 lines. Tests MUST reside in a separate `tests.rs` (or `tests/` directory), not inline in production files.

#### Scenario: Oversized modules are split into sub-features under 500 lines
- **WHEN** a module exceeds 500 lines
- **THEN** it is split into sub-feature directories so every `.rs` file is under 500 lines

#### Scenario: Sub-feature files use 1-word names
- **WHEN** a sub-feature directory is created under `engine/src/<feature>/<subfeature>/`
- **THEN** its source files use standardized 1-word names (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`, `tests.rs`) and omit files for roles the sub-feature does not use

#### Scenario: Tests may be co-located or separate
- **WHEN** inspecting any `.rs` file under `engine/src/`
- **THEN** tests may be inline via `#[cfg(test)]` or in a sibling `tests.rs`, at the author's discretion
## Why

Several workspace crates are already outgrowing flat single-file modules—`combat/src/lib.rs` is 947 lines, `extraction/src/lib.rs` is 450 lines, and `presentation/src/` mixes `visuals.rs` (357 lines), `hud.rs` (417 lines), and `menus.rs` (367 lines) alongside the newer `anim/` folder. Establishing a consistent, scalable layout before Phase 1–2 expansion keeps compilation fast, preserves the headless Simulation vs. Presentation boundary, and enforces the 500-line and concise 1-word naming rules in `AGENTS.md`.

## What Changes

- Standardize the two-layer workspace structure keeping `crates/sim/` (headless simulation feature crates) and `crates/presentation/` (visual/client concern crate) strictly separated.
- Adopt a **nested sub-feature** convention inside `crates/sim/<feature>/src/` (`weapon/`, `projectile/`, `reload/`, `pickup/`, `zone/`, `flow/`, `upgrade/`, etc.), each containing standardized 1-word files (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`) as needed so every file stays under 500 lines.
- Enforce a **two-tier event model**: cross-feature Bevy messages stay in `crates/data/src/events.rs` to prevent circular crate dependencies, while feature/sub-feature internal events live in `<subfeature>/event.rs`.
- Reorganize `crates/presentation/src/` at top level by visual concern (`camera/`, `input/`, `anim/`, `vfx/`, `audio/`, `scene/`, `ui/`), nesting feature sub-modules inside `scene/` (`arena.rs`, `player.rs`, `combat.rs`, `enemy.rs`, `extract.rs`) and `ui/` (`theme.rs`, `hud/`, `menu/`) to prevent monolith files.
- Refactor existing oversized modules (`sim/combat`, `sim/extraction`, `sim/enemies`, `sim/world`, and `presentation` including `anim/` module consistency) into the new structure without altering runtime gameplay behavior.
- Update `docs/07-technical-architecture-save-and-roadmap.md` (and `AGENTS.md` if needed for layout conventions) to record the finalized directory and module standard.

## Capabilities

### New Capabilities

- `project-layout`: Workspace crate layering (`data`, `sim/*`, `presentation`, `core`), nested sub-feature module conventions (`mod.rs`, `config.rs`, `state.rs`, `event.rs`, `system.rs`), concern-based `presentation` organization with feature sub-modules under `scene/` and `ui/`, two-tier event ownership, and file-size/naming invariants.

### Modified Capabilities

<!-- No existing spec-level behavior changes (`ron-config-loading` requirements remain unchanged). -->

## Impact

- **Simulation Crates (`game/crates/sim/`)**:
  - `combat/src/lib.rs` (947 lines) split into `weapon/`, `projectile/`, `reload/` (plus thin `lib.rs` plugin wiring).
  - `extraction/src/lib.rs` (450 lines) split into `pickup/`, `zone/`, `flow/`, `upgrade/`.
  - `enemies/src/` (`lib.rs`, `config.rs`, `tests.rs`) and `world/src/` (`arena.rs`, `player.rs`, `input_intent.rs`, `config.rs`) aligned to the sub-feature module convention.
- **Presentation Crate (`game/crates/presentation/src/`)**:
  - `visuals.rs` split into `scene/` (`arena.rs`, `player.rs`, `combat.rs`, `enemy.rs`, `extract.rs`).
  - `hud.rs` + `menus.rs` split into `ui/` (`theme.rs`, `hud/`, `menu/`).
  - `camera.rs` and `input.rs` moved into `camera/` and `input/` concern modules (with `vfx/` and `audio/` ready as Phase 1–2 placeholders/modules when added).
  - `anim/` reviewed/adjusted for consistency with the concern-folder conventions (including keeping `anim/mod.rs` under 500 lines and clean separation from `scene/player.rs`).
- **Core & Data (`game/crates/data/`, `game/core/src/main.rs`)**:
  - Public crate re-exports preserved or updated cleanly so `core/src/main.rs` and `ron-config-loading` continue working with zero gameplay regressions.
- **Docs**:
  - `docs/07-technical-architecture-save-and-roadmap.md` updated to reflect the refactored workspace tree.

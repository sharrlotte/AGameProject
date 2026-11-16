## Why

The current crate compiles after the 0.20 port but has no declared, readable
structure for its ECS execution order, several enum-tagged component types, and a
large amount of named-ordering debt hidden inside plugin scheduling. Future
slices (registry symbols, feature collapse, save redesign) become dramatically
safer and smaller if the foundation slice first establishes explicit simulation
sets, typed state boundaries, and removes enum-tag components that couple
unrelated systems together.

## What Changes

- Add a documented `SimulationSet`/`GameSet`-style ordering contract for the
  fixed and frame update pipelines, with all plugins wiring into named sets
  instead of ad-hoc `FixedUpdate`/`FixedFirst`/`FixedPostUpdate` chains.
- Replace `EntityType` (used as a shared component tag with three different
  meanings: spawn tag, selection key, lifetime marker) with purpose-built
  marker components and query filters; selection no longer keys off an enum.
- Split `GameVars` into typed resources (e.g., camera/follow state, deploy
  payload) so no single grab-bag resource is shared by unrelated systems.
- Delete dead re-exports and empty leftover modules; tighten visibility
  (`pub(crate)` where possible) across the crate.
- Sync `docs/architecture.md` and `docs/controls.md` with the new ordering
  contract and state layout. No behavior change is expected.

## Capabilities

### New Capabilities
- `simulation-sets`: declarative ordering sets and contract for the fixed/
  frame schedules (input → simulate → apply → cleanup), plus migration of all
  plugins to those sets.

### Modified Capabilities
<!-- No existing specs; this is the first change. -->

## Impact

- Code: `main.rs` (plugin wiring), `system.rs`, `constant.rs`, `content/*/`'s
  plugin registration, `scene/*`, `mission/mod.rs`, `save/io.rs` (imports
  only where shared types move), `resource.rs` (splitting `GameVars`),
  `content/entity.rs` (removed), `docs/architecture.md`, `docs/controls.md`.
- No schema/format changes, no new dependencies, no new gameplay behavior.

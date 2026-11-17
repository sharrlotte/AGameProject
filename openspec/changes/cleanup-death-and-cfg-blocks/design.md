## Context

- `Death` is currently a marker component inserted on destroyed blocks, bullets, missiles, expired lifetime entities, and destroyed ships, then despawned in `FixedLast` (`SimulationSet::Cleanup`) by `despawn_dead`.
- `#[cfg(debug_assertions)]`, `#[cfg(feature = "dev")]`, and `#[cfg(test)]` attributes are currently repeated on individual imports, structs, impls, and functions in 6 files.

## Goals / Non-Goals

**Goals:**
- Eliminate the `Death` marker component and `despawn_dead` system, using `commands.entity(entity).despawn()` directly at destruction sites.
- Group scattered `#[cfg(...)]` items into minimal `#[cfg(...)] mod ... { ... }` blocks across the 6 identified files.
- Keep all existing tests passing.

**Non-Goals:**
- Refactoring `main()` into smaller registration functions (separate TODO).
- Splitting `resource.rs` into `control/` and `ui/` folders (separate TODO).

## Decisions

### 1. Direct `despawn()` instead of `Death` marker
- Replace `commands.entity(e).insert(Death)` with `commands.entity(e).despawn()`.
- Remove `Without<Death>` from queries in `bullet/system.rs` and `ship/mutation.rs`.
- In `bullet/system.rs` where `(Visibility::Hidden, Death)` was inserted, despawn directly (or `despawn()` handles removal without needing `Visibility::Hidden`).
- Keep `ShipDestroyed` message in `content/death/component.rs` (or `DeathPlugin` registering `ShipDestroyed`) so ship destruction events remain intact.
- Update unit tests in `content/ship/mutation.rs` that previously asserted `contains::<Death>()` to assert `!app.world().entities().contains(ship_entity)` (or `get_entity(ship_entity).is_err()`).

### 2. Consolidate `#[cfg(...)]` into `mod` blocks
- For each of the 6 files (`system.rs`, `main.rs`, `scene/game.rs`, `content/ship/component.rs`, `content/ship/system.rs`, `content/block/feature/power/grid.rs`), group contiguous or related `#[cfg(...)]` imports, types, and functions into a single `#[cfg(...)] mod ...` block and re-export public items with `#[cfg(...)] pub use ...;` where needed.
- Inline `#[cfg(...)]` statements inside function bodies (e.g. `app.add_plugins(...)` in `main.rs`) remain as-is where they cannot be extracted into a module without changing control flow, or are grouped into a single `#[cfg(...)]` helper function.

## Risks / Trade-offs

- **Same-tick query visibility**: `commands.entity(e).despawn()` is deferred until the next command flush, so if two systems in the same schedule without an intervening `ApplyDeferred` both process `e`, the second system could still see `e` in its query. Mitigation: `SimulationSet::Death` already runs after `SimulationSet::Damage`; verify `cargo test` passes and no double-despawn warnings occur.

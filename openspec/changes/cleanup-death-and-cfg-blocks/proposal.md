## Why

Two codebase TODOs can be resolved cleanly together:
1. `core/src/content/death/component.rs` notes that the `Death` marker component may be unnecessary when entities can be despawned directly and death effects can listen to `ShipDestroyed`.
2. `core/src/system.rs` notes that scattered `#[cfg(...)]` attributes should be merged into fewer, more readable blocks—a pattern that also appears in `main.rs`, `scene/game.rs`, `content/ship/component.rs`, `content/ship/system.rs`, and `content/block/feature/power/grid.rs`.

## What Changes

- Remove the `Death` marker component and `despawn_dead` system; replace `commands.entity(entity).insert(Death)` with `commands.entity(entity).despawn()` and remove `Without<Death>` query filters across `block`, `bullet`, `lifetime`, `ship/mutation`, and `scene/game`.
- Keep `ShipDestroyed` message registration in `DeathPlugin` (or move it to `ship` if appropriate, keeping `DeathPlugin` minimal or removing it if `ShipDestroyed` is registered elsewhere).
- Consolidate scattered `#[cfg(debug_assertions)]`, `#[cfg(feature = "dev")]`, and `#[cfg(test)]` attributes into grouped `mod` blocks across:
  - `core/src/system.rs`
  - `core/src/main.rs`
  - `core/src/scene/game.rs`
  - `core/src/content/ship/component.rs`
  - `core/src/content/ship/system.rs`
  - `core/src/content/block/feature/power/grid.rs`
- Remove the resolved TODO comments in `core/src/content/death/component.rs` and `core/src/system.rs`.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `entity-lifecycle`: Entities destroyed in simulation are despawned directly via `Commands::despawn()` instead of being marked with `Death` for `FixedLast` cleanup.
- `code-organization`: Conditional compilation (`#[cfg(...)]`) blocks are grouped into minimal module blocks.

## Impact

- `core/src/content/death/*`
- `core/src/content/block/system.rs`
- `core/src/content/bullet/system.rs`
- `core/src/content/lifetime/system.rs`
- `core/src/content/ship/mutation.rs`
- `core/src/scene/game.rs`
- `core/src/system.rs`
- `core/src/main.rs`
- `core/src/content/ship/component.rs`
- `core/src/content/ship/system.rs`
- `core/src/content/block/feature/power/grid.rs`

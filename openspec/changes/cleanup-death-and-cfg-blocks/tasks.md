## 1. Remove `Death` component and `despawn_dead`

- [ ] 1.1 Remove `Death` struct and TODO comment from `core/src/content/death/component.rs`, keeping `ShipDestroyed`
- [ ] 1.2 Remove `despawn_dead` system from `core/src/content/death/system.rs` (keep `DeathPlugin` registering `ShipDestroyed`)
- [ ] 1.3 Replace `commands.entity(...).insert(Death)` with `commands.entity(...).despawn()` in `core/src/content/block/system.rs`, `core/src/content/lifetime/system.rs`, and `core/src/scene/game.rs`
- [ ] 1.4 Replace `Death` insertions and remove `Without<Death>` filters in `core/src/content/bullet/system.rs`
- [ ] 1.5 Replace `Death` insertions, remove `Without<Death>` filters, and update unit test assertions in `core/src/content/ship/mutation.rs`

## 2. Consolidate `#[cfg(...)]` blocks

- [ ] 2.1 Group `#[cfg(debug_assertions)]` items into a module in `core/src/system.rs` and remove the TODO comment
- [ ] 2.2 Group `#[cfg(debug_assertions)]`, `#[cfg(feature = "dev")]`, and `#[cfg(all(debug_assertions, feature = "dev"))]` items in `core/src/main.rs`
- [ ] 2.3 Group `#[cfg(debug_assertions)]` items in `core/src/scene/game.rs`
- [ ] 2.4 Group `#[cfg(debug_assertions)]` items in `core/src/content/ship/component.rs` and `core/src/content/ship/system.rs`
- [ ] 2.5 Group `#[cfg(test)]` items in `core/src/content/block/feature/power/grid.rs`

## 3. Verification

- [ ] 3.1 Run `cargo test` (and `cargo check --features dev`) to verify compilation and all unit tests pass

# Bevy 0.20 Migration Spec

## Goals

- Bump bevy 0.17.3 → 0.20.1 and all direct dependencies to the versions proven on the dev branch.
- Mechanical migration only; runtime behavior identical.

## Non-goals

- No UI behavior changes (button hover/press semantics preserved).
- No feature work, no code restructure beyond what the version bump requires.

## Version set (matches dev branch lockfile)

| Crate | Old | New |
|---|---|---|
| bevy | 0.17.3 | 0.20.1 |
| bevy-inspector-egui | 0.35 | 0.38.0 |
| bevy_rapier2d | 0.32.0 | 0.37.0 |
| bevy_hanabi | 0.17 | 0.20.0 |
| bevy_brp_extras (`dev` feature) | 0.17.3 | 0.23.0 |
| rand | 0.9 | 0.10.3 |
| serde | 1.0.228 | 1.0.229 |
| serde_json | 1.0.145 | 1.0.154 |
| uuid | 1 | 1.29.2 |
| directories | 6.0.0 | unchanged |

`game/Cargo.toml` workspace root and `game/.cargo/config.toml` unchanged (no dependencies there).

## API mapping

| 0.17 | 0.20 |
|---|---|
| `Button` + `Changed<Interaction>` + `With<Button>` | `bevy::picking::hover::PickingInteraction` (auto-inserted by picking; `Changed` fires on hover/press transitions). `Button` is not constructible (alias to a private struct). |
| `Interaction::{Pressed,Hovered,None}` | `PickingInteraction::{Pressed,Hovered,None}` |
| `BorderRadius::all(...)` as separate component | `Node::border_radius` field |
| `TextFont { font_size: f32 }` | `font_size: FontSize::Px(f32)` |
| `Velocity { linvel, angvel }` | `Velocity { linear, angular }` |
| `intersect_shape(..., \|entity\| {...})` | callback is `FnMut(Entity, &RapierCollider) -> bool` |
| `Cuboid::new(Vector2::new(w, h))` (nalgebra) | `Cuboid::new(Vec2::new(w, h))` (glam) |
| `RapierContextInitialization::InitializeDefaultRapierContext` | requires `broad_phase_optimization_strategy` |
| `RapierConfiguration { ... }` | requires `simulation_mode` and `num_threads` |
| `on_timer(..).and(c)`, `in_state(..).or(c)` | `SystemCondition` combinators renamed: `and_then`, `or_else` |
| `use rand::Rng` (`random_range`/`random_bool`) | `use rand::RngExt` |

## Touch points

- `game/core/Cargo.toml`, `game/Cargo.lock`
- `game/core/src/main.rs` (rapier init)
- `game/core/src/scene/{game,pause,home,setting,editor}.rs` (UI buttons, FontSize, border_radius)
- `game/core/src/scene/background.rs`, `game/core/src/utils/random.rs` (rand)
- `game/core/src/content/{bullet,ship,damage}/systems.rs` (rapier shapes, Velocity, FontSize, rand)
- `game/core/src/content/block/radar/systems.rs`, `content/block/feature/efficiency/systems.rs` (intersect_shape)

## Verification

- `cargo check --features dev` passes with 0 errors; remaining warnings pre-existing.
- Manual pass pending: `cargo dev` — menu hover/click, ship editor block placement, HUD update, pause menu.

## Out of scope

- Deprecated `bevy::ui::Interaction` path (still functional but removed next release).
- Bevy feature collections refactor (`default-features = false`).
- Rewriting UI buttons to observer handlers (dev branch idiom, `On<PointerClick>`).

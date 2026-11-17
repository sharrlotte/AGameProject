use bevy::prelude::*;

/// Ordering contract for gameplay systems.
///
/// The target schedule map (implemented across phases 0–3) is:
///
/// | Schedule        | Set       | Contents                                                 |
/// |-----------------|-----------|----------------------------------------------------------|
/// | `PreUpdate`     | `Input`   | keyboard/mouse, UI buttons, pause toggle, editor input     |
/// | `FixedPreUpdate`| `Stats`   | power, efficiency, thrust/rotate stats, cargo aggregation  |
/// | `FixedUpdate`   | `Ai`      | AI think/steering at `ai_think_rate_hz`                    |
/// | `FixedUpdate`   | `Control` | player forces, camera follow target                        |
/// | `FixedUpdate`   | `Weapons` | turret targeting/tracking, shooting, projectiles, radar    |
/// | `FixedUpdate`   | `Physics` | Rapier step (engine)                                       |
/// | `FixedPostUpdate`| `Damage` | collision events → damage events → health                  |
/// | `FixedPostUpdate`| `Death`  | block destruction → `ShipMutation` → drops                 |
/// | `PostUpdate`    | `Present` | camera, HUD, indicators, damage text                       |
/// | `Last`          | `Cleanup` | deferred despawns                                          |
///
/// Damage → Death → `ShipMutation` is chained explicitly inside one fixed tick;
/// message persistence across frames is never assumed.
#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum SimulationSet {
    Input,
    Simulate,
    Apply,
    Cleanup,
}

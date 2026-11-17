use bevy::prelude::*;

/// Ordering contract for gameplay systems.
///
/// Each variant is a system set used to express intra-schedule ordering. The
/// same set may appear in more than one schedule; ordering is configured per
/// schedule in [`crate::main`].
///
/// | Schedule         | Set       | Contents                                              |
/// |------------------|-----------|-------------------------------------------------------|
/// | `PreUpdate`      | `Input`   | keyboard/mouse, UI buttons, pause toggle, editor input |
/// | `FixedFirst`     | `Stats`   | `PowerSet::{Generate, Boost, Demand, Solve}`          |
/// | `FixedPreUpdate` | `Stats`   | thrust/rotate stats, ship stat aggregation, cargo      |
/// | `FixedUpdate`    | `Ai`      | AI think/steering at `AI_THINK_RATE_HZ`                |
/// | `FixedUpdate`    | `Control` | player forces, camera follow target, drop collection   |
/// | `FixedUpdate`    | `Weapons` | turret targeting/tracking, shooting, projectiles, radar|
/// | `FixedUpdate`    | `Cleanup` | lifetime countdown                                     |
/// | `FixedPostUpdate`| `Damage`  | collision events → damage events → health, lasers      |
/// | `FixedPostUpdate`| `Death`   | block destruction → `ShipMutation` → drops             |
/// | `PostUpdate`     | `Present` | camera, HUD, indicators, damage text, parallax         |
/// | `Last`           | `Cleanup` | deferred despawns                                      |
///
/// `Damage → Death → ShipMutation` is chained explicitly inside one fixed
/// tick; message persistence across frames is never assumed.
#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum SimulationSet {
    /// Player input, UI buttons, and editor interactions.
    Input,
    /// Derived per-ship/block statistics (power, efficiency, thrust, cargo).
    Stats,
    /// AI behavior selection and steering.
    Ai,
    /// Player-controlled ship forces and camera target updates.
    Control,
    /// Turret tracking, shooting, projectiles, and radar scans.
    Weapons,
    /// Damage events, health mutation, and laser hit resolution.
    Damage,
    /// Block destruction, ship mutations, and world drops.
    Death,
    /// Camera, HUD, indicators, and other frame presentation.
    Present,
    /// Lifetime countdown and deferred despawns.
    Cleanup,
}

# AI Spec — Block-Introspective Utility AI & Team Intel

## Authoring model

Ships require no inline AI rule lists in RON. `ShipDesign` carries only an optional high-level `Doctrine` (`Auto | Brawler | Skirmisher | Sniper | Support | Escort`, defaulting to `Doctrine::Auto`):

```rust
pub enum Doctrine {
    Auto,
    Brawler,
    Skirmisher,
    Sniper,
    Support,
    Escort,
}
```

Numeric difficulty curves, feature unlock thresholds, radar baseline, bounded intel list caps (`bounds`), role/board/combat tuning (`role_tuning`, `board_tuning`, `combat_tuning`), and utility weights live in one global hot-reloadable config asset: `core/assets/config/base/ai.ron` (`AiConfig` / `AiConfigValue`).

Each ship receives `Skill(u8)` (`1..=10`), resolved on mission spawn in precedence:
`ShipSpawn.ai_level` -> (`ai_cfg.default_ally_level` if `team.is_player()`, else `MissionDef.ai_level` -> `GameSetting.ai_level` -> `ai_cfg.default_enemy_level`), clamped to `1..=10`.

Editor AI tab exposes `Doctrine` selector buttons (`Auto`, `Brawler`, `Skirmisher`, `Sniper`, `Support`, `Escort`) persisted on `ShipDesign`.

## Module layout

```
core/src/content/ship/ai/
├── mod.rs          AiPlugin, chained FixedUpdate SimulationSet::Ai systems
├── component.rs    Doctrine, Skill, Role, Capability, Readiness, Sensor, Brain, Tactic, Steering, FireOrder, TacticalBoard, TeamPlan, TeamIntel
├── config.rs       AiConfig, AiConfigValue, validation, hot-reload bridge
├── capability.rs   Pure Feature introspection + live Readiness + DamageEvent quadrant HP refresh
├── sensor.rs       max_radar_range scanning, TeamIntel sharing, DamageEvent alerts, decaying memory
├── board.rs        Per-team TeamPlan (focus_target, vip_ally, rally_point)
├── scorer.rs       Modular Utility Scorers + hysteresis + Booster power-shedding
└── steer.rs        Tactic -> Steering + FireOrder + shared debug gizmo helpers
```

## Runtime pipeline (`FixedUpdate`, `SimulationSet::Ai`)

1. `init_missing_capabilities` / `rebuild_capabilities_on_change` / `refresh_quadrant_hp_on_damage` — Pure `Feature` introspection builds `Capability` on spawn, `ShipChanged`, and `DamageEvent` (fixed `turn_speed == 0.0` vs. rotating turrets, `Rot90` mount angles, `LaserBeam` / `PullingBeam` / `EnemyBullet` PD, energy/ammo dependencies, `Booster` aura, 4-quadrant armor HP, `max_radar_range`, mobility, and inferred `Role`). Dedicated support hulls are auto-detected under `Doctrine::Auto` when `support_weight > offensive_dps * support_auto_ratio`.
2. `update_readiness` — Computes live `Readiness` (`hp_ratio`, `energy_ratio`, `ammo_ratio`, `disarmed`, `firing_drain_beam`, `power_supply`, `battery_ratio`). Loot drops remain player-only.
3. `update_sensors_and_intel` — Scans enemy ships within each ship's live `Capability.max_radar_range`. Any sighting immediately publishes a `SharedContact` into that team's `TeamIntel.contacts` (always active at all `Skill` levels `1..=10`). Lost living contacts persist in `TeamIntel.last_known` for `memory_secs`, and unseen `DamageEvent` hits publish shared `TeamAlert` bearings.
4. `update_tactical_board` — Updates `TacticalBoard` per-team `TeamPlan` (`rally_point`, `vip_ally`, and `focus_target`).
5. `evaluate_utility_and_power` — At `Skill`-scaled `think_rate_hz`, runs composable `Scorer`s (`Retreat`, `Recharge`, `Support`, `Engage`, `Regroup`, `Investigate`, `Search`) with `hysteresis` to update `Brain.current`. When `skill >= power_management_level`, toggles `Disabled` on owned `Booster` blocks during brownouts and restores them on recovery.
6. `steer_ships` — Translates `Brain.current` into `Steering { desired_heading, thrust_world, throttle_cap }` and `FireOrder { primary_target, lead_accuracy }`:
   - `SpinalAim` aligns fixed weapons at any `Rot90` mount to the target lead point.
   - High-skill ships unlock strafe evasion (`strafe_evade_level`), perpendicular spinal dodging (`spinal_dodge_level`), 4-quadrant armor angling (`armor_angling_level`), and drain-beam throttle budgeting (`power_management_level`).
   - `Regroup` / `Investigate` / `Search` / `Support` use a stateless ship-aware arrival controller. With `decel = ShipStats.thrust_power / ShipStats.mass`, the desired speed is the smaller of the braking-limited `sqrt(2 × decel × (dist - arrival_radius))` and the ship's own terminal speed `decel / linear_damping`; thrust is proportional to speed error, with negative thrust braking. Inside `arrival_radius` thrust is zero and `throttle_cap` is `0.0`; otherwise `throttle_cap` is `1.0`. There is no non-combat throttle-cap configuration.
   - Combat turrets (`Closest`, `MostHP`) prefer `FireOrder.primary_target` when in range and scale target velocity by `FireOrder.lead_accuracy`; `EnemyBullet` point-defense turrets remain autonomous.

Ships actively controlled by the player (`ControlledShip.controlling == Some(entity)`) are skipped by AI evaluation and steering.

## Movement application (`FixedUpdate`, `SimulationSet::Control`)

`update_ship_steering` aligns ship rotation via PD torque toward `Steering.desired_heading` (or thrust heading when `None`) and applies `Steering.thrust_world * thrust_power * throttle_cap`. Ships at or above `omnidirectional_thrust_level` use full omnidirectional thrust (matching player controls); ships below that threshold project thrust onto `local_y()`.

## Player vision

Player fog-of-war is driven solely by the player team's shared intel in `TacticalBoard.teams[DEFAULT_TEAM_ID].intel`. `VisionPlugin` (`content/ship/vision/`, `PostUpdate` / `SimulationSet::Present`) filters with `With<Ship>` and sets every non-player ship's `Visibility` to `Visible` when its entity is present in player `TeamIntel.contacts`, and to `Hidden` otherwise. Player-team ships are always `Visible`. Ship blocks and visual layers use `Visibility::Inherited` so hiding the `Ship` hides all hull/turret sprites, while bullets, missiles, and laser beams stay `Visible` regardless of shooter detection.

The same predicate gates anti-leak surfaces: offscreen edge indicators (`draw_indicators`), drag selection, dev ship click inspection (`With<Ship>`), and dev ship debug gizmos skip hidden/undetected enemies. `TeamIntel.last_known` and `TeamIntel.alerts` do not reveal the ship entity or render on the minimap.

## Camera assist

`ControlMode::Pilot` follows the controlled ship; `ControlMode::FreeLook` lets WASD pan the camera. `F` toggles the mode and middle-mouse drag pans in either mode. Zoom is via the mouse wheel between `min_zoom` and `max_zoom`.

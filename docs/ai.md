# AI Spec — Per-Ship Inline Profiles

## Authoring model

AI is data, not Rust marker components. Each `ShipDesign` carries an inline
`AiProfile` (`content/ship/ai/component.rs`):

```rust
pub struct AiProfile {
    pub initial: BehaviorSpec,
    pub rules: Vec<AiRule>,
}
pub struct AiRule {
    pub priority: i32,
    pub when: AiCondition,
    pub then: BehaviorSpec,
    pub cooldown_secs: f32,
}
```

`AiCondition` is one of `Always`, `HpBelow`, `HpAbove`, `EnemyInRange`,
`AllyCountBelow`, `AllyCountAbove`, `TargetLost`. `BehaviorSpec` selects one of
`Idle`, `Wander`, `Patrol`, `Guard`, `Follow`, `Regroup`, `Kite`, `Chase`,
`Attack`, `Flee` with a `TargetRef`/`PositionRef` where needed.
All of these derive serde and round-trip through RON.

Editor presets (`Aggressive`, `Guard`, `Escort`, `Skirmisher`) copy
a complete profile into the inline field; the AI tab shows a RON preview and
validation errors before saving.

## Runtime

`ShipDesign::spawn` inserts `AiProfile` and `AiRuntime` on the ship entity.
`AiRuntime` holds a single `AiBehavior` enum (the old 11 `BehaviorX` markers,
`BehaviorTimer`, `ShipBehavior`, and `AIShipController` are gone), the spawn
position, leader, per-rule cooldowns, and a think timer.

`AiPlugin` runs three chained systems in `FixedUpdate`, `SimulationSet::Ai`:

1. `build_ship_index` — rebuilds per-team and enemy snapshots (`ShipIndex`).
2. `evaluate_ai_rules` — at the shared `AI_THINK_RATE_HZ`, ticks cooldowns, tests conditions,
   and applies the highest-priority matching rule. Ties keep the current
   behavior. Wander points re-roll on arrival.
3. `apply_ai_behaviors` — converts the current `AiBehavior` into a
   `ShipBehavior` steering intent consumed by `update_ship_behavior`.

## Behavior semantics

- `Idle` — hold position.
- `Wander` — re-rolled point around the spawn.
- `Patrol`/`Regroup` — approach a position (spawn, squad center, leader).
- `Guard`/`Follow` — stay within radius of the resolved ally/target.
- `Kite` — retreat when closer than the configured range, orbit otherwise.
- `Chase`/`Attack` — approach to `ai_min_range_factor × weapon range`, then
  orbit; `Attack` reacquires the nearest enemy when its target is lost.
- `Flee` — move **away** from the threat until `distance` is reached (the old
  inverted-sign bug is fixed).

Targets are invalidated when the entity no longer has a core or leaves radar
range; the next think tick reacquires via `TargetRef`.

Ships followed by the camera (`CameraTarget.following`) or controlled by the
player (`ControlledShip.controlling`) are skipped by AI.

## Movement application

`PostUpdate::update_ship_behavior` turns `ShipBehavior::Approach`/`GoAround`
into `ExternalForce` with a braking model based on `linear_damping`.

## Camera assist

`ControlMode::Pilot` follows the controlled ship; `ControlMode::FreeLook` lets
WASD pan the camera. `F` toggles the mode and middle-mouse drag pans in either
mode. Zoom is via the mouse wheel between `min_zoom` and `max_zoom`.

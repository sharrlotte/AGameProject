## 1. AI Configuration & Difficulty Plumbing

- [x] 1.1 Create `core/assets/config/base/ai.ron` with `default_enemy_level`, `default_ally_level`, continuous `Skill(1..=10)` curves (`think_rate_hz`, `lead_accuracy`, `aim_jitter_rad`, `range_precision`, `evasion_strength`, `memory_secs`), feature unlock thresholds, and role/scoring tuning weights
- [x] 1.2 Implement `AiConfig`, `AiConfigValue`, validation (`AiConfig::validate`), and hot-reload bridge in `core/src/content/ship/ai/config.rs` and register asset loading in `core/src/config/`
- [x] 1.3 Add `ai_level: u8` (`1..=10`, default `5`) to `GameSetting` (`core/src/config/setting.rs` & `core/assets/config/setting.ron`) and optional `ai_level: Option<u8>` to `MissionDef` and `ShipSpawn` (`core/src/mission/def.rs`)

## 2. Core AI Components & Block Introspection

- [x] 2.1 Rewrite `core/src/content/ship/ai/component.rs` with `Doctrine`, `Skill`, `Role`, `Capability`, `Readiness`, `Sensor`, `Brain`, `Tactic`, `Posture`, `SupportKind`, `Steering`, `FireOrder`, `TacticalBoard`, `TeamPlan`, `TeamIntel`, `SharedContact`, `LastSeen`, and `TeamAlert`, removing legacy `AiProfile`, `AiRule`, `AiCondition`, `BehaviorSpec`, `TargetRef`, `PositionRef`, `AiBehavior`, `AiRuntime`, `PRESETS`, and `ShipIndex`
- [x] 2.2 Implement pure `Feature`-driven live introspection in `core/src/content/ship/ai/capability.rs` (fixed vs. rotating weapons & `Rot90` mounts, weapon archetypes, DPS-weighted ranges, energy/ammo dependencies, `Booster`/support blocks, 4-quadrant armor HP, `max_radar_range`, auto/override `Role`, and live `Readiness`) recomputed on spawn and `ShipChanged`

## 3. Team-Wide Intel & Tactical Board

- [x] 3.1 Implement `core/src/content/ship/ai/sensor.rs` for live `max_radar_range` scanning, always-on team-wide `TeamIntel` contact publishing (all `Skill` levels), shared `DamageEvent` `TeamAlert` recording, and decaying `last_known` team memory
- [x] 3.2 Implement `core/src/content/ship/ai/board.rs` to compute per-team `TeamPlan` (`focus_target`, `vip_ally`, `rally_point`) each tick in `SimulationSet::Ai`

## 4. Modular Utility AI, Power Shedding & Steering

- [x] 4.1 Implement modular Utility `Scorer`s with hysteresis in `core/src/content/ship/ai/scorer.rs` (`Engage`, `Recharge`, `Retreat`, `Support`, `Regroup`, `Investigate`, `Search`) and `Skill`-gated active `Booster` power-shedding (`Disabled` insert/remove)
- [x] 4.2 Implement `core/src/content/ship/ai/steer.rs` translating `Tactic` + `Capability` + `Skill` into decoupled `Steering { desired_heading, thrust_world, throttle_cap }` (spinal lead aim, strafe-kite, spinal-line dodging, 4-quadrant armor angling, drain-beam throttle budgeting) and `FireOrder { primary_target, lead_accuracy }`, plus shared debug geometry helpers
- [x] 4.3 Wire chained `SimulationSet::Ai` systems in `core/src/content/ship/ai/mod.rs` and replace `ShipBehavior` / `update_ship_behavior` with `Steering` / `update_ship_steering` in `core/src/content/ship/{component.rs, system.rs, mod.rs}` (progressive omnidirectional thrust unlocked by `Skill`)
- [x] 4.4 Integrate `FireOrder` into `update_turret_target` (focus target + `Skill`-scaled lead accuracy for combat turrets; `EnemyBullet` PD autonomous) and fix `turret_tracking` to track `state.aim_point` in `core/src/content/block/feature/turret/system.rs`

## 5. Ship Designs, Save Schema & UI Updates

- [x] 5.1 Bump `DESIGN_SCHEMA_VERSION = 3`, replace `ShipDesign.ai` with `#[serde(default)] pub doctrine: Doctrine`, resolve `Skill` on ship spawn in `ShipDesign::spawn_with_id` / mission spawn, and migrate all 4 starter `.ron` files (`interceptor`, `raider`, `b2`, `blackbird`)
- [x] 5.2 Bump `SAVE_SCHEMA_VERSION = 7` in `core/src/save/battle/` and update `ShipSnapshot` / `WorldSave` capture & restore for `Doctrine`, `Skill`, `BrainSave`, and `TeamIntel`
- [x] 5.3 Add `AI Skill (1..10)` `[-]` / `[+]` stepper to `core/src/scene/setting/mod.rs` and replace `core/src/scene/editor/{ai_panel.rs, ui.rs, mod.rs, save.rs}` rule/preset UI with `Doctrine` selector buttons
- [x] 5.4 Update dev ship debug UI text and gizmos to read `Brain`, `Skill`, `Doctrine`, `Capability`, and `Steering` instead of removed `AiRuntime` and `ShipBehavior`

## 6. Documentation & Verification

- [x] 6.1 Update `/docs/ai.md`, `/docs/config.md`, `/docs/save.md`, and `/docs/ui.md` specs to match the new AI architecture, `base/ai.ron`, schema versions, and UI
- [x] 6.2 Add/update unit tests covering `AiConfig` validation, `Capability` introspection (fixed vs. rotating, block loss, support ratio), `TeamIntel` sharing & decaying memory, Utility scorers & hysteresis, `Booster` power-shedding, `Steering` & `FireOrder` / `turret_tracking`, and v3 design / v7 battle save round-trips
- [x] 6.3 Run targeted `cargo test` suites and launch `cargo dev` in the background to verify clean startup and runtime behavior

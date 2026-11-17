## Context

The `core` crate is a Bevy 0.20.1 + Rapier 2D game. Ships are made of grid blocks with per-block health, turret, ammo, thruster, and storage features. The current implementation grew in slices: `ShipGrid` has two parallel maps (`blocks: HashMap<IVec2, Entity>` and `cells: HashMap<IVec2, BlockData>`), a separate `Blocks` component holds a second block list, saves are split between `world.ron` and `ships/*.ron`, and block identity is duplicated across `Block`, `BlockData`, `BlockState`, `BlockSaveData`, and `SchematicBlockData`. Several slices also left overlapping marker components and placeholder implementations.

Observed defects driving this change:

- Destroying a block updates `Blocks` but leaves its cells in `ShipGrid`; the collider is rebuilt from the grid, so destroyed blocks still collide.
- `Ship.id` is taken from an arbitrary first block; multiple cores overwrite each other.
- Bounds are initialized to `0` and `BoundingBox` is never updated, so negative-coordinate ships clamp incorrectly.
- `WorldSave.elapsed_secs` stores `started_at`, ships restore at the origin, and velocity/mission/drops/AI/camera are not saved.
- Power has no producers in `block.ron`; `PowerConsumer`/`PowerStorage` are unused; efficiency is ship-wide rather than per consumer.
- Turrets target `blocks[0]`; `MostHP` and `EnemyBullet` are placeholders; empty barrels divide by zero; pulling-beam strength scales with hit block count.
- Continuous laser and pulling-beam visuals are parented to the non-rotating turret block while hitboxes use the rotating barrel, so the visual points elsewhere.
- The AI flee direction is inverted, several behaviors are unreachable, and attack can target `Vec2::ZERO`.
- Manual `SceneEntity` cleanup leaks transient entities; player control mixes camera panning and flight.
- The editor is one 1,419-line file, always edits the first ship, and autosaves on every edit.
- The background spawns 2,720 sprites, damage text is spawned per hit, and radar/debug UI rebuild every fixed tick.

Constraints: keep the game playable after every phase; no new renderer/physics stack; Bevy 0.20 APIs; use RON for all game data; no production `unwrap()` or unsafe; single source of truth per concept; tests accompany each phase.

## Goals / Non-Goals

**Goals:**

1. One canonical representation per concept at each lifecycle stage (placement, runtime grid entry, block entity identity, ship stats, save snapshot).
2. A physically correct ship model: destruction removes all occupied cells, detaches and deletes core-less fragments, updates bounds/stats, and rebuilds the collider from live cells only.
3. A real save system: five RON slots, each a self-contained full snapshot with embedded ships, strict version checks, atomic writes, and a hard cutover from old formats.
4. A working power loop with producers, consumers, equal-share efficiency, feature scaling, validation, and a minimal HUD.
5. Complete targeting: core targets, real `MostHP`, point defense against bullets/missiles, lead prediction, safe empty barrels, and aligned laser visuals.
6. Per-ship AI profiles with prioritized rules, every behavior reachable, runtime state saved, and an editor panel.
7. Shared cargo with pooled capacity and atomic pickup.
8. Leak-free state transitions and one documented schedule contract.
9. No config or mission panic path; validation reports readable errors.
10. Remove repetitive save/config/stat code through a small set of traits and macros.
11. Remove the largest performance costs and split the editor into maintainable modules.

**Non-Goals:**

- New missions, campaign progression, or narrative.
- Multiplayer/networking.
- New art beyond a reactor placeholder and starter-ship layouts.
- Deterministic replay (RNG state is saved, but no replay log).
- Saving in-flight bullets, missiles, lasers, or beams; they are cleared on load.
- A general modding API or reflection-based serializer.
- Rewriting the renderer or physics stack.
- Ammo reload from cargo (cargo is capacity/collection in this pass).
- Migrating old JSON schematics or v1 world saves.

## Decisions

### 1. Keys are `String`, not interned symbols

Delete `src/symbol.rs`, `Symbol`, `Symbols`, and the `lasso` dependency. `BlockDef.block_type`, `BlockRegistry` keys, `ItemDef.item_type`, `AmmoConfig.crafting`, and all RON fields use `String`.

Rationale: `Symbol` already wraps an owned `String`; `Symbols::intern` touches a `Rodeo` but discards the key, so the interner provides no deduplication. It adds a dependency and a resource threaded through every loader with no benefit. Config maps already use `String` keys. If profiling later shows clone cost, switch to `Arc<str>`; do not reintroduce an interner.

Alternative rejected: keep `lasso` and make the interner real. The game's key counts are tiny and config/UI paths need `&str` anyway.

### 2. Ship model: `ShipGrid` is canonical for placement and identity

Canonical types:

```rust
pub struct BlockPlacement {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: String,
    pub rotation: u8,
}

pub struct GridBlock {
    pub placement: BlockPlacement,
    pub entity: Entity,
}

#[derive(Component, Default)]
pub struct ShipGrid {
    pub blocks: HashMap<Uuid, GridBlock>,
    pub cells: HashMap<IVec2, Uuid>,
}

#[derive(Component)]
pub struct BlockRef { pub id: Uuid }

#[derive(Component)]
pub struct BlockOwner { pub ship: Entity }
```

- `ShipGrid` owns occupied cells and the block id/type/rotation. Block entities carry only `BlockRef` and `BlockOwner` plus runtime state (health, ammo, turret state, timers).
- Delete `BlockData`, `Block`, `BlockState`, `Blocks`, `ShipData`, `ShipState`, `ShipCalculatedStat`, `ShipCalculatedData`, `ShipFragment`, and `ComputedShipStats`.
- `Ship.id` is the core block id. `Ship.core_position` is the core block center in ship-local space.
- `ShipStats` is the single stats component and includes mass, ranges, health/max_health, power fields, cargo fields, thrust, and rotate speed.
- `BoundingBox` has `min`, `max`, `size`, `center`, initialized from `i32::MAX`/`i32::MIN`, and recomputed on every mutation.

All placement changes go through one pipeline:

```rust
pub enum ShipMutation {
    DestroyBlock { ship: Entity, block_id: Uuid },
    DestroyBlocks { ship: Entity, block_ids: Vec<Uuid> },
    PlaceBlock { ship: Entity, block: BlockPlacement },
    RebuildFromLayout { ship: Entity, layout: Vec<BlockPlacement> },
}
```

`apply_ship_mutations` runs once per fixed tick, batches by ship, and orders work as:

1. Remove destroyed blocks from `ShipGrid` (all occupied cells).
2. Recompute connected components from `cells`.
3. Find the component containing the core.
4. Remove every block in a core-less fragment from the grid, mark its entity `Death`, and emit drops for enemy ships.
5. If the core is gone, mark the ship `Death` and skip collider rebuild.
6. Recompute `Ship`, `BoundingBox`, and `ShipStats`.
7. Rebuild the collider from `ShipGrid.cells` only.
8. Emit `ShipChanged { ship }` for HUD/save/AI invalidation.

`BlockRegistry::stats_from_grid` iterates `grid.blocks()`, validates exactly one core, sets `Ship.id` from the core, computes bounds from cells, computes mass/health/power/cargo/thrust aggregates, and sets turret ranges (`max_range` or `0.0`, `min_range` or `0.0`). No core means the entity is a fragment, never a ship.

Spawn and editor paths both use `ShipGrid::rebuild_from_layout` followed by `validate_ship`: exactly one core, no overlapping cells, all cells inside block shapes, no disconnected non-core components at spawn, no empty turret barrels, valid power/storage values.

Alternative rejected: keep `Blocks` as a read cache. It duplicates identity and placement and is the source of the fragment bug; a single grid plus derived stats is simpler and fast enough at this scale.

### 3. Scene lifecycle and schedules

Replace `SceneEntity(SceneState::X)` and `clean_state_item` with `DespawnOnExit`:

- `SceneState::Game` on gameplay roots and transient combat visuals.
- `SceneState::Home` on home UI.
- `SceneState::ShipEditor` on editor UI.
- `PauseState::Paused` on pause UI.
- `SettingState::Open` on settings UI.

Children despawn with parents, so no manual cleanup system is needed. Delete `SceneEntity`, the `Save` marker, and `ShipState`. Add a debug-only audit that counts `DespawnOnExit`-tagged roots versus spawned gameplay roots and warns about untagged entities.

Schedule contract:

| Schedule | Set | Contents |
|---|---|---|
| `PreUpdate` | `Input` | keyboard/mouse, UI buttons, pause toggle, editor input |
| `FixedPreUpdate` | `Stats` | power, efficiency, thrust/rotate stats, cargo aggregation |
| `FixedUpdate` | `Ai` | AI think/steering at `ai_think_rate_hz` |
| `FixedUpdate` | `Control` | player forces, camera follow target |
| `FixedUpdate` | `Weapons` | turret targeting/tracking, shooting, projectile movement, radar |
| `FixedUpdate` | `Physics` | Rapier step (engine) |
| `FixedPostUpdate` | `Damage` | collision events → damage events → health |
| `FixedPostUpdate` | `Death` | block destruction → `ShipMutation` → drops |
| `PostUpdate` | `Present` | camera, HUD, indicators, damage text |
| `Last` | `Cleanup` | deferred despawns |

Chain `Damage → Death → ShipMutation` explicitly; do not rely on message persistence across frames. Remove the old `SimulationSet::Simulate` placement in `PreUpdate`/`PostUpdate`.

Load flow: `PendingLoad { slot }` → `NextState<SceneState>::Loading` → exiting `Game` despawns automatically → entering `Game` spawns from the snapshot instead of `spawn_mission` → restore battle/camera/AI/cargo/power → clear `PendingLoad`.

Control modes: `Pilot` (WASD flies the controlled ship, camera follows) and `FreeLook` (WASD pans, ship control paused). `F` toggles follow; middle-mouse drag pans in either mode. Default `Pilot` when a ship is controlled, `FreeLook` otherwise. This removes the current W/S conflict.

### 4. Save/load v2: RON, five slots, embedded ships

Paths:

```
<save_dir>/designs/*.ron
<save_dir>/saves/slot_1.ron ... slot_5.ron
```

Old `schematics/*.json` and `world/` are ignored and untouched. On first run, copy bundled starter designs from `assets/config/starter/*.ron` to `designs/` if empty.

Design schema:

```rust
pub struct ShipDesign {
    pub version: u32,
    pub id: Uuid,
    pub name: String,
    pub blocks: Vec<BlockPlacement>,
    pub ai: AiProfile,
}
```

World schema:

```rust
pub const SAVE_SCHEMA_VERSION: u32 = 2;
pub const SAVE_SLOTS: usize = 5;

pub struct WorldSave {
    pub version: u32,
    pub slot: u8,
    pub saved_at_unix_ms: u64,
    pub battle: BattleSave,
    pub camera: CameraSave,
    pub ships: Vec<ShipSnapshot>,
    pub drops: Vec<DropSave>,
    pub rng: RngSave,
}

pub struct ShipSnapshot {
    pub id: Uuid,
    pub team: u8,
    pub transform: TransformSave,
    pub velocity: VelocitySave,
    pub ai: AiProfile,
    pub ai_runtime: AiRuntimeSave,
    pub blocks: Vec<BlockSnapshot>,
}

pub struct BlockSnapshot {
    pub placement: BlockPlacement,
    pub health: f32,
    pub max_health: f32,
    pub turret: Option<TurretState>,
    pub ammo: Option<AmmoState>,
    pub single_pattern: Option<SingleShootPatternState>,
    pub burst_pattern: Option<BurstShootPatternState>,
    pub laser_pattern: Option<LaserBeamPatternState>,
    pub pulling_pattern: Option<PullingBeamPatternState>,
}
```

- `BattleSave` contains mission name, elapsed battle seconds, and optional outcome.
- `WorldSave` embeds every ship; loading never opens per-ship files and there is no `ship_ids`.
- Projectiles, lasers, beams, damage text, and transient VFX are not serialized; they are cleared on load.
- RNG is a seeded `GameRng` resource stored in the snapshot.
- `TurretState.target` is skipped during serialization; targets are reacquired after load.
- Drops store position, item type, amount, and remaining lifetime.

Atomic write: pretty RON to `slot_N.ron.tmp`; if `slot_N.ron` exists, rename it to `slot_N.ron.bak`; rename `.tmp` to `slot_N.ron`. On any error, keep the old file and report through UI.

Hard cutover: no JSON reader, no v1 world reader, no phantom fabrication, no "repair ships with ≤2 blocks". Missing designs are simply absent; the UI offers "New Ship".

Starter content: `interceptor.ron` (core, hull, thruster, laser turret; Aggressive) and `raider.ron` (core, hull, thruster, swarmer; Skirmisher). They validate under `validate_ship` and double as test fixtures.

Alternative rejected: per-ship files plus a world index. It reintroduces id indirection, partial saves, and multi-file atomicity for no practical benefit at this game size.

### 5. Power: producers, consumers, equal share

`PowerProducer { output: f32 }` and `EnergyConsumer { rate: f32 }` are both serde config types and runtime components; no duplicate config/runtime structs. Delete unused `PowerConsumer` and `PowerStorage`.

Content: core produces `10.0`; new reactor produces `25.0`; laser turret consumes `5.0`, swarmer `4.0`, spectre `8.0`, thruster small `3.0`. Values are starting points.

Formula:

```text
production = sum(PowerProducer.output)
consumers  = blocks with EnergyConsumer.rate > 0
demand     = sum(consumer.rate)

if consumers is empty:        efficiency_i = 1.0
else if production >= demand: efficiency_i = 1.0
else:
    share = production / consumers.len()
    efficiency_i = clamp(share / demand_i, 0.0, 1.0)
```

`PowerEfficiency` stores per-block efficiency. `ShipStats.power_production`, `power_demand`, `power_used`, and `efficiency` expose aggregates; `power_used = sum(demand_i * efficiency_i)`, ship efficiency is `power_used / power_demand`.

Effects: at `efficiency <= 0.01`, powered features are inactive. Turret `effective_fire_rate = base_fire_rate * efficiency`; pattern state advances only while active. Thruster force and torque are multiplied by efficiency. Non-consuming blocks are unaffected.

`EfficiencyProvider`/`EfficiencyEffect` (radius buff) is renamed `EfficiencyAura`/`EfficiencyBuff`; final multiplier = power efficiency × aura buff.

HUD (bottom-left, controlled ship):

```
POWER  10 / 18      EFF 55%
[██████░░░░]
```

Green when production ≥ demand, orange for 1–99%, red for 0% or consumers without a producer. Updates on `ShipChanged`/`PowerChanged`, not every frame. Editor stats panel shows the same numbers.

### 6. Targeting, point defense, and the laser direction fix

**Laser bug.** Continuous laser and pulling-beam hitboxes use the rotating barrel's `GlobalTransform`, but the beam visual is spawned as a child of the turret block, which does not rotate with the barrel. Fix:

- Parent the beam visual to `barrel_ref.0` (the `TurretSprite` entity), not the turret block.
- Keep the local transform `(0.0, range / 2.0, LAYER_BULLET)` so the beam extends along barrel-local +Y.
- Do the same in `handle_pulling_beam`.
- In the projectile `Laser` path, change the sprite center offset from `Vec2::new(width / 2.0, length / 2.0)` to `(0.0, length / 2.0)`.
- Regression test: beam parent is the barrel; beam local direction and hitbox direction both equal barrel forward after rotation.

**Target point.** All ship strategies aim at the target ship's core, not `blocks[0]`. If the core is missing, the target is invalid.

**Closest.** Nearest enemy core within range; tie-break by entity id.

**MostHP.** Highest current ship HP within range, using cached `ShipStats.health`; tie-break by distance, then entity id.

**EnemyBullet.** Add `ProjectileOwner { team }` and `Interceptable` to `Bullet` and `Missile`; `Laser` is not interceptable. The turret queries hostile `Interceptable` projectiles within range, selects nearest (missiles prioritized when close), and stores `TurretTarget::Projectile(Entity)`. Aim with lead prediction using projectile velocity and the turret projectile speed; fall back to current position if no solution. On hit, despawn the projectile; a missile is destroyed without triggering its blast.

`TurretState { target: Option<TurretTarget>, aim_point: Vec2, can_fire: bool }`. Fire gate checks barrel forward versus `aim_point`, not block position.

Other fixes: empty barrels validated and runtime-guarded (no `% 0`); pulling beam applies pull once per target ship per tick; damage text uses `LAYER_DAMAGE_TEXT = 400.0`; missile parent/visual z and hitbox hierarchy aligned; fire/cooldown timers reset on spawn and pause correctly; transient combat visuals state-scoped.

### 7. AI: per-ship inline profiles

```rust
pub struct AiProfile {
    pub think_rate_hz: f32,
    pub initial: BehaviorSpec,
    pub rules: Vec<AiRule>,
}

pub struct AiRule {
    pub priority: i32,
    pub when: AiCondition,
    pub then: BehaviorSpec,
    pub cooldown_secs: f32,
}

pub enum AiCondition {
    Always,
    HpBelow(f32),
    HpAbove(f32),
    EnemyInRange(f32),
    AllyCountBelow(u32),
    AllyCountAbove(u32),
    TargetLost,
}

pub enum BehaviorSpec {
    Idle,
    Wander { radius: f32 },
    Patrol { around: PositionRef, radius: f32 },
    Guard { target: TargetRef, radius: f32 },
    Follow { target: TargetRef, radius: f32 },
    Regroup { position: PositionRef, radius: f32 },
    Kite { range: f32 },
    Chase { target: TargetRef },
    Attack { target: TargetRef },
    Flee { from: TargetRef, distance: f32 },
    Kamikaze { target: TargetRef },
}

pub enum TargetRef { NearestEnemy, HighestHpEnemy, NearestAlly, Leader, SpawnPoint, SquadCenter }
pub enum PositionRef { SpawnPoint, SquadCenter, Leader, SelfCore }
```

Profiles are inline in `ShipDesign.ai`. Editor presets copy values into the inline profile: `Aggressive`, `Guard`, `Escort`, `Skirmisher`, `Kamikaze`.

Runtime `AiRuntime { current: AiBehavior, target: Option<Entity>, spawn_position: Vec2, leader: Option<Entity>, rule_cooldowns: Vec<f32> }` replaces the 11 `BehaviorX` markers and `ShipBehavior`. One `AiBehavior` enum (`Idle`, `MoveTo`, `Attack`, `Flee`, ...) is the current runtime behavior. Rules are evaluated at `think_rate_hz` in `FixedUpdate`; highest matching priority wins; current behavior continues if no rule matches; cooldowns prevent thrash; targets are invalidated when dead or out of radar.

Behaviors: `Idle` no force; `Wander` random point around spawn, re-roll on arrival; `Patrol` orbit/approach a point; `Guard` stay near target, engage inside radius; `Follow` stay within radius of target ally; `Regroup` move to squad center/leader; `Kite` keep preferred range from nearest enemy and fire while retreating; `Chase` approach until weapon range; `Attack` face target, approach to `ai_min_range_factor × weapon range`; `Flee` move **away** until distance is reached (fix inverted sign); `Kamikaze` full thrust toward target and apply a configurable blast on contact, reusing the missile blast helper.

Alternative rejected: keep the marker-component FSM. It makes rules and per-ship authoring impossible without editing Rust, and the markers overlap with a controller enum.

### 8. Cargo: shared per ship

Cargo lives in the existing `ShipStats` component (`cargo_capacity`, `cargo_used`, `cargo_items`); there is no separate `ShipCargo` component. Capacity is the sum of `StorageConfig.capacity` over all storage blocks in `ShipGrid`. Delete `Storage`, `StorageBuilder`, and `StorageState`; storage blocks only contribute capacity. Pickup is atomic: if the full drop amount does not fit, it is not collected and remains in the world. Only player ships collect drops in this pass. `ShipChanged` recomputes capacity when storage blocks are added or destroyed. `collect_drop` finds the nearest player ship with space, magnets toward it, and despawns only on successful add. HUD shows `CARGO 120 / 1000` plus an item breakdown in pause/editor stats.

Alternative rejected: per-block inventories. They duplicate capacity and make pickup non-atomic across blocks.

### 9. Config validation and errors

Replace panics with `ConfigState::Error(ConfigError)` and an error screen. Validate at load:

- Unique block keys; every `FeatureConfig` has a runtime `Feature`.
- Core exists and its group is `"core"`.
- Turret `barrels` non-empty; `range > 0`; `turn_speed >= 0`; `fire_rate > 0`; burst `amount > 0`.
- `PowerProducer.output >= 0`; `EnergyConsumer.rate > 0`; storage `capacity > 0`.
- Shape `size > 0`; voxels within size; no duplicate voxel positions.
- Mission references existing designs; player/enemy teams valid.
- Starter designs: one core, connected, no overlap.
- AI profiles: valid conditions/behaviors, priorities, cooldown ≥ 0.
- Team ids < 16; replace `assert!` in `Team::new` with a validated `Result` constructor and config-level checks.

Mission load failure returns to Home with a message instead of `panic!`.

### 10. One type per concept

| Concept | Canonical type | Removed / merged |
|---|---|---|
| Block placement | `BlockPlacement { id, position, block_type: String, rotation }` | `BlockData`, `SchematicBlockData`, placement fields of `BlockSaveData`, `Block` |
| Runtime grid entry | `GridBlock { placement, entity }` | `ShipGrid.cells: HashMap<IVec2, BlockData>` |
| Block identity | `BlockRef { id }`, `BlockOwner { ship }` | `Block`, `BlockState` |
| Block config | `BlockConfig` → `BlockDef` after asset load | keep split only for loaded handles |
| Ship design | `ShipDesign { version, id, name, blocks, ai }` | `SchematicData`, design half of `ShipData` |
| Runtime ship | `Ship` + `ShipStats` + components | `ShipState`, runtime half of `ShipData`, `ShipSave` |
| Save file | `WorldSave { ..., ships: Vec<ShipSnapshot> }` | `WorldSave.ship_ids`, `ships/*.ron`, separate `SlotSave` |
| Block snapshot | `BlockSnapshot { placement, ... }` | `BlockSaveData`, `TurretSaveState` |
| Turret | `Turret { strategy, pattern, turn_speed, range, barrels }` + `TurretState` | `TurretComponent` mechanics; visual fields stay in config |
| Ammo | `AmmoConfig { capacity, crafting, icon }` + `AmmoState { current }` | `Ammo`, `AmmoBuilder` |
| Storage | `StorageConfig { capacity }` + `ShipStats` cargo fields | `Storage`, `StorageBuilder`, `StorageState`, `ShipCargo` component |
| Power | `PowerProducer { output }`, `EnergyConsumer { rate }` | `PowerConsumer`, `PowerStorage`, config/runtime duplicates |
| Ship stats | `ShipStats` | `ComputedShipStats`, `ShipCalculatedStat`, `ShipCalculatedData`, proposed `ShipHealth`/`ShipPower`/`ShipCargo` split |
| Fragments | `ShipBuildResult { ship, fragments }` | `ShipFragment`, `ShipCalculatedStat` wrapper |
| AI | `AiProfile`, `AiRuntime`, `AiBehavior` | 11 `BehaviorX` markers, `ShipBehavior` |
| Drops | `ItemDrop { item_type, amount }`, `DropSpawn { position, item }` | `DropComponent` single-variant enum, `DropSave` |
| Scene marker | `DespawnOnExit` | `SceneEntity`, `ShipState`, `Save` |
| Symbol | `String` | `Symbol`, `Symbols`, `lasso` |

Config/runtime pairs that stay: `BlockConfig`/`BlockDef`, `FeatureConfig`/`Feature`, `ShipDesign`/`Ship`, `AiProfile`/`AiRuntime`; their fields differ materially (asset paths vs handles, static vs live state).

### 11. Traits and macros

Keep the abstraction layer small and explicit:

```rust
pub trait RonDocument: Serialize + DeserializeOwned {
    const VERSION: u32;
    const KIND: &'static str;
}

pub struct Versioned<T> {
    pub version: u32,
    pub data: T,
}

pub trait CaptureSnapshot {
    fn capture(world: &World) -> Self;
}

pub trait RestoreSnapshot {
    fn restore(self, commands: &mut Commands, registry: &BlockRegistry);
}
```

- `read_document`, `write_document`, and `atomic_write` are the only save/design IO paths; they own version checks, RON parse errors, `.tmp`/`.bak` handling.
- `ron_document!` implements `RonDocument` for `WorldSave` and `ShipDesign`.
- `CaptureSnapshot`/`RestoreSnapshot` are implemented for `WorldSave`, `ShipSnapshot`, `BlockSnapshot`, and `ItemDrop`; `save_world`/`load_world` become thin orchestration. Do not build a reflection serializer.
- `feature_set!` generates `FeatureConfig`/`Feature` enums, `From<FeatureConfig> for Feature`, runtime insertion, and validation. Adding a feature becomes one line.
- Extend the existing `make_stat_system!` with `make_ship_stat!` and a generated registration helper so new stats do not need a plugin edit. Power/efficiency use the same pipeline.

Alternative rejected: proc-macro derives. The repetitive cases are enumerable with `macro_rules!`; a proc-macro crate adds build cost and hides control flow.

### 12. Performance and editor structure

Performance:

- Replace 2,720 background sprites with one procedural WGSL starfield or 2–3 tiled parallax layers updated only when the camera moves. Fallback: keep sprites but update only on camera movement.
- Pool 64 `Text2d` damage numbers; reuse instead of spawn/despawn per hit.
- Rebuild radar at 10 Hz or on `ShipChanged`, not every tick.
- Put debug ship UI behind a `DebugOverlay` resource toggle and update on change.
- Make ammo warning icons children of blocks or update on change; no per-tick reposition.
- Sort `BlockRegistry::iter` once at registration.

Editor split:

```
scene/editor/mod.rs        plugin, state, shared types
scene/editor/ui.rs         panels, layout constants
scene/editor/input.rs      placement, deletion, camera
scene/editor/ship_list.rs  design list, selection
scene/editor/save.rs       RON save/load, debounce, messages
scene/editor/ai_panel.rs   AI profile editor
scene/editor/grid.rs       grid drawing/preview
```

One `ShipDesignChanged` message replaces `ShipSaveChanged`/`ShipSavesChanged`. Autosave is debounced 500 ms plus an explicit Save. `selected_ship_id: Option<Uuid>` replaces "first ship". Panel widths are named constants (220/250). Play/Deploy builds a fresh payload from the current grid and validates it; no stale `DeployPayload`. Settings save is debounced and merged with defaults.

## Risks / Trade-offs

- [Grid-canonical refactor touches most systems] → Delete parallel stores first; compiler-driven migration; grid/stats/mutator tests before porting consumers.
- [Full snapshot becomes a swamp] → `WorldSave` embeds ships; pure capture/restore traits; RON round-trip tests; projectiles explicitly out of scope.
- [Hard cutover loses player ships] → Explicit decision; old files untouched; starter designs ensure fresh installs are playable; one-time UI notice.
- [State-scoped migration misses spawns] → Debug audit system; grep for `commands.spawn`; entity-count tests per scene.
- [Equal-share power starves high-draw weapons] → HUD makes it visible; values tunable; later priority tiers do not require schema changes.
- [Per-ship AI editor is a large UI] → Presets first; rule list add/remove/reorder; validate and round-trip before polish.
- [Macro/trait layer over-abstracts save IO] → Keep only `RonDocument`, `Versioned<T>`, `CaptureSnapshot`, `RestoreSnapshot`; no reflection serializer.
- [Type consolidation breaks many call sites] → Phase 0/1 deletions are compiler-guided; no behavior change in Phase 0; tests per merge.
- [`String` clones cost more than interning] → Acceptable at current scale; switch to `Arc<str>` only if profiling proves it.
- [Bevy 0.20 API differences] → Verify `DespawnOnExit` compiles in Phase 2; wrap if the API shifts.
- [Laser fix regresses hitbox] → Parent/direction test plus visual playtest; hitbox remains barrel-based.
- [Migrating old saves is impossible] → Old formats are deliberately ignored; document the break in `docs/save.md` and the UI.

## Migration Plan

Each phase must compile, pass its tests, and leave the game playable; do not start the next phase until the current one is green.

| Phase | Scope |
|---|---|
| 0 — Baseline, simplification, guardrails | Remove `lasso`/`Symbol`/`Symbols` and JSON paths; add `RonDocument`, `Versioned<T>`, atomic RON IO, `ConfigState::Error`, error screen; delete dead power types; add test baseline and schedule documentation. |
| 1 — Ship model v2 | Add `BlockPlacement`, `GridBlock`, `BlockRef`, `BlockOwner`, `ShipStats`, `ShipBuildResult`; rewrite `ShipGrid`; delete parallel block/ship stores; add `ShipMutation`; stats/bounds/collider from live grid; update spawn/editor. |
| 2 — Scene lifecycle and schedules | Replace `SceneEntity` with `DespawnOnExit`; implement schedule map; chain damage → death → mutation; split pilot/camera; add audit. |
| 3 — RON save v2 | Add `ShipDesign`/`WorldSave`/`ShipSnapshot`/`BlockSnapshot`; remove per-ship files and JSON; five slots; full snapshot; starter designs; save/load UI; capture/restore traits; hard cutover. |
| 4 — Power | Add producer/consumer types, reactor content, equal-share efficiency, feature scaling, power HUD, editor stats, validation, feature/stat macros. |
| 5 — Targeting, combat, laser fix | Core targets, `MostHP`, point defense, `TurretState`, laser parenting/offset, empty barrels, pulling beam, damage text, missile hierarchy, timing. |
| 6 — AI profiles | Add schema/runtime, rule evaluation, all behaviors, flee/attack/wander fixes, kamikaze blast, `FixedUpdate`, editor AI panel, starter profiles. |
| 7 — Shared cargo | Add cargo fields to `ShipStats`, pooled capacity, atomic pickup, cargo HUD, remove per-block storage. |
| 8 — Performance and editor split | Background reduction, damage-text pool, radar cadence, debug overlay, editor modules, settings debounce, remaining review fixes. |

Rollback: each phase is a separate commit; revert the phase commit. Save schema v2 is a hard cutover, so rolling back Phase 3+ means the v2 files are unreadable by the old build; keep a pre-rewrite copy of the game data directory if old saves matter.

## Open Questions

None blocking. These are tunable during implementation without changing the design: exact power values, starter layouts/names, kamikaze damage/radius, AI preset thresholds, HUD placement/colors, slot labels, and camera/ship keybinds.

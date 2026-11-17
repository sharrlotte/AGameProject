# Simulation Spec — Ships and Blocks

## Blocks

### Grid and shape

- Grid cell = `GRID_SIZE = 64.0` px. Blocks occupy a set of grid cells from
  `BlockShape::filled(w, h)` or `BlockShape::default()` (1×1).
- Block anchor = bottom-left grid cell. World placement from
  `BlockData::to_transform`: `position * GRID_SIZE + pixel_size/2`.
- Each voxel is flagged with `connectable_faces` (`FACE_RIGHT`/`LEFT`/`UP`/`DOWN`),
  default `FACE_ALL`. Placement validity (editor) requires face-match connectivity
  to the existing grid.
- Multi-cell blocks: voxels are filled rectangle; no complex-shape support yet
  (`// TODO: Better shape data type…`).

### Block definition (`BlockDef`)

| Field | Meaning |
|---|---|
| `name` | Display name |
| `block_type` | Key; id string (`hull`, `core`, …) looked up in `BlockRegistry` |
| `block_group` | `Core`/`Base`/`Turret` — sort order in the editor palette |
| `mass`, `health` | Used by ship stat computation and `Health` |
| `shape: BlockShape` | Size + voxels |
| `cost: u32` | Not currently enforced anywhere |
| `features: Vec<Feature>` | Components inserted directly on the block entity; turret barrel is the one render-only child |
| `drops: Vec<DropComponent>` | Drop events when block is destroyed |

Registry: `BlockRegistry` (resource, `HashMap<Symbol, BlockDef>`), populated
from the keyed `block.ron` map at config load. Duplicate ids rejected at boot.

### Block registry contents (as of review)

| BlockType | Name | Group | Mass | HP | Cost | Voxel shape | Features | Drops |
|---|---|---|---|---|---|---|---|---|
| Hull | Standard Hull | Base | 10 | 1000 | 5 | 1×1 | Texture `block/surge-wall-large.png` | Steel ×10 |
| ThrusterSmall | Small Ion Thruster | Base | 10 | 5000 | 50 | 1×1 | Texture `block/thruster.png`, Thruster(200, 100) | Steel ×10 |
| Core | Core | Core | 50 | 5000 | 150 | 3×3 | Texture `block/core-nucleus.png` + team overlay, Thruster(200, 60), Storage(1000) | Steel ×10 |
| LaserTurret | Laser Turret | Turret | 7 | 1600 | 150 | 1×1 | Texture `block/block-2.png` + `block/lancer.png`, Turret(…) | Steel ×10 |
| BlockScrap | Scrap Block | Base | 10 | 1800 | 200 | 1×1 | Texture `block/block-iron.png` | Steel ×10 |
| BlockSteel | Steel Block | Base | 8 | 1600 | 200 | 1×1 | Texture `block/block-steel.png` | Steel ×10 |
| Swarmer | Swarmer Turret | Turret | 8 | 1800 | 150 | 1×1 | Texture + `swarmer.png`, Turret(…) | Steel ×10 |
| Spectre | Spectre Turret | Turret | 16 | 3600 | 150 | 3×3 | Texture + `spectre.png`, Turret(…) | Steel ×10 |
| Meltdown | Meltdown Turret | Turret | 20 | 3600 | 150 | 3×3 | Texture + `meltdown.png`, Turret(…) | Steel ×10 |
| Foreshadow | Foreshadow Turret | Turret | 50 | 13600 | 150 | 3×9 | Texture + `foreshadow.png`, Turret(…) | Steel ×10 |
| Colaris | Colaris Turret | Turret | 50 | 13600 | 150 | 1×1 | Texture + `colaris.png`, Turret(…) | Steel ×10 |
| Parallax | Parallax Turret | Turret | 50 | 13600 | 150 | 1×1 | Texture + `colaris.png` (duplicated), PullingBeam(…) | Steel ×10 |

`BlockType::Meltdown` is the spelled-wrong enum variant name used everywhere.

### Block rendering

Blocks spawn with all feature components on the same entity: `Block`, `BlockState`,
`Health`, `BlockOwner`, `Team`, drop `Collider` (cuboid, `Sensor`, mass 0) in a
per-team hitbox layer (`Group::from_bits_truncate(1 << (16 + team_id))`),
plus `Sprite`/`Thruster`/`Storage`/`EnergyConsumer`/`EfficiencyProvider`/`Turret`
as configured. The only child entity is a turret's rotating barrel
(`TurretSprite` + `Transform` + `Sprite`); the block carries `TurretBarrel`
pointing to it. Extra texture layers beyond the first also remain render-only
children so their z-ordering is preserved. Text labels and flash overlays
are children of the block.

### Block destruction

- `Health` is decremented from `DamageEvent`s (see combat.md).
- `handle_block_destroy` runs in `FixedPostUpdate`; when `health <= 0` it inserts `Death`
  and emits `BlockDestroyedEvent`. Drops are written for **non-player** teams only.
- Core destruction kills the whole ship: inserts `Death` on the ship entity.
- `check_rebuild_ship_collider` (ship module) then:
  1. Removes the destroyed block from `Blocks` list.
  2. Recomputes connectivity (`BlockRegistry::blocks_stats`) into a new ship fragment set.
  3. Blocks in fragments without a Core get `Death` inserted and drop as loot.
  4. The remaining ship gets a fresh voxel collider and `ColliderMassProperties::Mass`.

### Fragmentation

`blocks_stats` performs a flood fill over the per-ship `ShipGrid` cache
(`IVec2 → BlockData` in `ShipGrid.cells`), populated once on ship spawn and updated
incrementally on block destroy (`grid.remove_block`). No full `HashMap` rebuild
happens on the normal hot path; `ShipGrid::rebuild` is the correctness fallback used
at spawn/load. The grid splits into (a) one ship and (b) zero or more fragments.

## Ships

### Spawn pipeline

`ShipData::spawn` (schematic/live data):
1. `BlockRegistry::blocks_stats` → ship data (mass, core position, range) + fragments.
   Empty core requirement: blocks with no Core → return `None` (not spawned).
2. Spawn ship entity with `RigidBody::Dynamic`, voxel collider, collision group
   `GROUP_SHIP`, `ActiveEvents::COLLISION_EVENTS`, `AIShipController::new(Idle)`,
   `Damping { LINEAR_DAMPING = 0.9, ANGULAR_DAMPING = 0.999 }`.
3. Spawn `Blocks` child blocks (via `BlockData::spawn`), each inserting
   `BlockOwner { ship }` plus all feature components directly on the block.
4. `ShipUIRoot` child is attached holding a `Text2d` debug dump rebuilt every tick
   (FPS + camera coords + behavior/stats).

### Coordinate/anchor semantics

Ship entity origin = world origin; `Ship.core_position` is the core offset in *world* px.
`BLOCK` children are positioned by `def.to_transform(position)` (core anchored).
`BoundsBox.size = size.as_vec2() * GRID_SIZE`. Rotations are z-axis; forward = local +Y
(`transform.local_y()`).

### Movement

- Player control: `ExternalForce` in `ship/system.rs::player_control_ship`.
  - `W/↑` forward thrust, `S/↓` reverse, `A/D` turn (torque ≈ `rotate_speed`),
    `Q/E` strafe. Shift = camera speed ×3 (camera only).
  - W/S apply force along `forward * thrust_power`.
- AI control: `PostUpdate::update_ship_behavior` applies `ExternalForce` + torque from
  `ShipBehavior` (`Approach` / `GoAround`) computed from `AIShipBehavior` state
  transitions in `FixedUpdate::control_ship`. Ships never change `ExternalForce`
  outside these systems.
- `Damping` variants come from `constant.rs` (`LINEAR_DAMPING = 0.9`,
  `ANGULAR_DAMPING = 0.999`).

### Drops

- `DropEvent` on `FixedUpdate` is converted to `Lifetime::seconds(DROP_DESPAWN_IN_SECONDS = 300)`
  pickups (`DROP_SIZE = 10`).
- `collect_drop`: drops within `DROP_COLLECT_RADIUS = 1000` of the *player* ship are
  attracted toward it; within `sqrt(DROP_COLLECT_RADIUS) ≈ 31.6` px they are consumed into
  the player's storage if not full.
- Non-player block destruction emits drops; player destruction deletes blocks silently.

### Team model

`Team { id: u8 }` — `DEFAULT_TEAM_ID = 0` is the player; `Team::new` asserts `id < 16`.
Color derived from palette index. Collision groups:

| Group | Used by | Filter includes |
|---|---|---|
| `GROUP_1` (ship) | ship hull collider | `GROUP_SHIP` |
| `GROUP_2` (bullet) | bullets, missile hitboxes | enemy block-hitbox bits (16..31) minus own |
| `GROUP_3` (radar) | Radar sensors | `GROUP_SHIP` |
| per team bit 16+id | block colliders | `GROUP_BULLET` |

Bit indices 16–31 hold per-team block hitboxes; up to 16 factions.

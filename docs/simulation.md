# Simulation Spec — Ships and Blocks

## Canonical types

- `BlockPlacement` — static block description (`id`, `position`, `block_type`, `rotation`).
- `GridBlock` — one live block in a grid (`placement` + `entity`).
- `BlockRef` / `BlockOwner` — identity and ownership components on block entities.
- `BlockGrid { blocks: HashMap<Uuid, GridBlock>, cells: HashMap<IVec2, Uuid> }` —
  authoritative placement and identity for ships and debris. `cells` is derived from
  `blocks` on every `rebuild_from_layout` (including collider rebuilds).
- `ShipStats` — single aggregate per ship (mass, health, ranges, power, cargo,
  thrust). Derived by `BlockRegistry::stats_from_grid` on every `ShipChanged`.

The old `BlockData`, `Block`, `BlockState`, `Blocks`, `ShipData`, `ShipState`,
`ShipCalculated*`, and `ComputedShipStats` types are deleted.

## Blocks

Grid cell = `GRID_SIZE = 64.0` px. A block occupies the cells of its baked `Footprint`
for its placement `Rot90`; placement validity in the editor requires a shared orthogonal
edge (Manhattan distance 1) with another block. Shapes are authored as `ShapeSpec`
(`Square`, `Rect`, `Triangle`, `Rows`) and baked once at config load into four
`(Footprint, Hitbox)` pairs — one per placement rotation. `core` is `Square(3)`-sized
(`Rect((3,3))`), `spectre` `Rect((3,3))`, `foreshadow` `Rect((3,9))`; triangles and
`Rows` masks are supported for non-rectangular footprints.

`BlockDef` fields: `name`, `block_type`, `block_group`, `mass`, `health`,
`shape`, `cost`, `features: Vec<Feature>`, `drops`. `BlockRegistry` (resource)
maps string ids to defs, rejects duplicates, and keeps an `iter()` order sorted
once at registration by block group.

Features (`FeatureConfig`/`Feature`, shared between RON config and runtime):
`Texture`, `Turret`, `Generator`, `Battery`, `Consumer`, `Booster`, `Thruster`,
`Storage` (capacity only). Power components are shared between serde and runtime.

Block entities carry `BlockRef`, `BlockOwner`, `Health`, `Team`, the placement
translation and `Rot90` rotation, a sensor `Collider` built from the baked hitbox in a
per-team hitbox layer, and all feature components (components stay on the block entity so
save/load reads them directly). A `Visual` child applies the definition rotation and flips
as signed scale; textures and the rotating turret barrel (`TurretSprite`) are its children.
Art is authored once for the base shape with alpha matching the true shape, so one texture
serves all four definition rotations and the collider never sees negative scale. Turret
aim compensates for a mirrored `Visual` so barrels point at targets in world space.

Hitboxes follow the shape: `Square`/`Rect` → cuboid, `Triangle` → convex polygon of the
true triangle, `Rows` → compound of per-cell cuboids, each in block-local pixel space
centered on the footprint bounding box for that rotation. The ship physics collider is a
`Collider::compound` of the baked hitboxes of live blocks, positioned at each block's
footprint center; it replaces the old voxel-point collider.

## Power

Each ship is one isolated power grid solved in `FixedFirst` (`SimulationSet::Stats`)
via chained `PowerSet::{Generate, Boost, Demand, Solve}`:

1. **Generate**: `Generator { output }` contributes `output × Boost` (previous tick's boost).
   `Battery { capacity }` pools stored energy (`Charge`, spawns full, lost on block destruction).
2. **Boost**: `Booster { radius, boost, rate }` takes priority power before consumers,
   ordered by ascending distance to the ship core. A booster is binary: fully powered or off.
   Powered boosters apply `Boost` (max stacking, own team within physics radius on any ship).
3. **Demand**: Active consumers report `Demand`:
   - Thrusters with `Consumer { rate }`: `rate × max(throttle.linear, throttle.angular) × Boost`.
   - Charge turrets (`Single`/`Burst`): `charge_rate × Boost` while `TurretCharge < capacity`, else `0`.
   - Beam turrets (`LaserBeam`/`PullingBeam`): `rate` while firing, else `0`.
4. **Solve**: Remaining available power (`generation + stored / dt - booster_draw`) splits
   proportionally across active demand: `supply = clamp(available / demand, 0, 1)` (`1` when `demand == 0`).
   Net surplus charges batteries up to `capacity`; net deficit discharges batteries before `supply` drops.
   `Supply ≤ 0.01` (`STARVATION_THRESHOLD`) disables thrust and beams.

`ShipStats` exposes `power_production`, `power_demand`, `power_supply`, `power_stored`, `power_capacity`.
Movement intent is tracked per ship by `Throttle { linear, angular }` (`0..1`), written by both
player control and AI steering.

## Destruction and fragmentation

`handle_block_destroy` writes `BodyMutation::DestroyBlock { body, block_id }` for blocks
at `health <= 0`; only the mutation removes the block entity, its grid cells, and its
collider parts. `apply_body_mutations` (FixedPostUpdate, `Death` set) batches per body:
remove blocks, recompute connected components from Manhattan-1 adjacency of baked
footprint cells, detach every component that does not contain the core into a `Debris`
body, kill the ship and wreck every survivor on core loss, rebuild ship stats/bounds and
each body's compound collider from live baked hitboxes, emit drops, and send
`ShipChanged` for ships.

## Debris

A `Debris` body is a first-class inert grid body: `Debris`, its own `BlockGrid`, `Team`,
`RigidBody::Dynamic`, compound collider, mass, `Velocity`, `Lifetime`, and
`GROUP_DEBRIS`. Blocks keep `Health` and stay damageable. Destroying a debris block
splits the body recursively: the largest component keeps the entity and the rest become
new debris bodies. Debris never carries `Ship`, `ShipStats`, `AiRuntime`, or `Throttle`,
so ship list, AI, HUD, power, turret, cargo, and mission systems ignore it.

Ship components detached on destruction get a random lifetime in the configured
`debris_lifetime_min_seconds..max_seconds` range; splits inherit the parent's remaining
time. On expiry a non-player body emits each remaining block's `def.drops` at its world
position and despawns; player debris yields nothing. Expiry applies no damage or impulse.
Debris collides with ships and other debris, radar ignores it, and bodies never merge.

## Ships

`ShipDesign::spawn` builds a `BlockGrid` from a layout (using each placement's rotated
footprint), derives `GridBuildResult`, spawns the ship with `RigidBody::Dynamic`,
compound hitbox collider, `ShipStats`, `BlockGrid`, `AiProfile`, `AiRuntime`,
`DespawnOnExit(SceneState::Game)`, and spawns each block entity with its identity. The
debug `ShipUIRoot` shows behavior, stats, and pooled cargo.

Movement: player control uses `ExternalForce` (WASD/QE, Pilot mode); AI uses
`update_ship_behavior` steering intents from `AiBehavior`. Damping is configured
in `game.ron` (`linear_damping = 0.9`, `angular_damping = 0.999`).

## Drops and cargo

Cargo lives only on `ShipStats` (`cargo_capacity`, `cargo_used`, `cargo_items`),
pooled from all storage blocks. `collect_drop` attracts drops to the nearest
player ship and picks up atomically: the full amount must fit or the drop stays
in the world. `ShipChanged` recomputes capacity; losing storage discards the
excess.

## Teams

`Team { id: u8 }` — id 0 is the player, `Team::new` asserts `id < 16`.
`GROUP_SHIP`, `GROUP_BULLET`, `GROUP_RADAR`, and per-team bits 16+id implement
hitbox filtering for up to 16 factions.

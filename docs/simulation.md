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
edge where **both** sides are physically filled. `BlockDef::solid_edge` derives this from
the baked hitbox: square, rect and rows edges are always solid, while a triangle accepts
connections only on its filled leg edges — right triangles connect on two adjacent cell
edges (`R0` left+bottom, `R90` right+bottom, `R180` right+top, `R270` left+top), and
isosceles triangles only on their base. `validate_ship` rejects any adjacent occupied
cells that touch on an empty edge, and debris splitting uses the same rule. Shapes are authored as `ShapeSpec`
(`Square`, `Rect`, `Triangle`, `Rows`) and baked once at config load into four
`(Footprint, Hitbox)` pairs — one per placement rotation. `core` is `Square(3)`-sized
(`Rect((3,3))`), `spectre` `Rect((3,3))`, `foreshadow` `Rect((3,9))`; triangles and
`Rows` masks are supported for non-rectangular footprints. Five hull-only triangle
definitions ship as `wedge_1x1` (`Triangle(Right, (1,1))`), `cap_1x1`
(`Triangle(Isosceles, (1,1))`), `plate_2x2` (`Triangle(Right, (2,2))`), `fin_1x2`
(`Triangle(Isosceles, (1,2))`) and `spike_1x3` (`Triangle(Isosceles, (1,3))`).
Their polygon hitboxes match their alpha, and their mass, health, cost and steel
drops are 60% of a same-cell-count square, per size.

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

## Starter designs

Four starter `ShipDesign` documents are embedded via `include_str!` in
`save/design/lib.rs` and written to the user data designs directory on every run
before the directory is loaded into `ShipCatalog`. `quick-battle.ron` selects them
by name: `Interceptor` for the player, `B2` and `Blackbird` for enemies.

Every starter uses one 3×3 `core` at grid cell `(-1, -1)`. Layouts are connected,
non-overlapping, and satisfy `validate_ship`. Composition:

| Design | Parts | Structure | Weapons | Support |
|---|---|---|---|---|
| `Interceptor` | 39 | 15 `hull`, 10 `wedge_1x1`, 1 `spike_1x3` | 3 `laser_turret` | 4 `thruster_small`, 2 `reactor`, 2 `battery`, 1 `booster` |
| `Raider` | 49 | 23 `hull`, 12 `wedge_1x1`, 1 `spike_1x3` | 3 `swarmer` | 4 `thruster_small`, 2 `reactor`, 2 `battery`, 1 `booster` |
| `B2` | 56 | 32 `hull`, 12 `wedge_1x1`, 2 `plate_2x2` | 2 `colaris`, 1 `foreshadow` | 2 `thruster_small`, 2 `reactor`, 2 `battery` |
| `Blackbird` | 16 | 3 `hull`, 1 `wedge_1x1`, 2 `fin_1x2`, 1 `spike_1x3` | 1 `laser_turret` | 4 `thruster_small`, 1 `booster`, 1 `reactor`, 1 `battery` |

The interceptor is a narrow arrow with an isosceles nose spike and wedge leading
edges; the raider is a broad swept wedge with turrets at the wingtips and nose.
`B2` is a broad flying wing with two `plate_2x2` wingtips and serrated wedge leading
and trailing edges; it has 32 hull blocks and only 2 thrusters. Alongside its two
`colaris` it carries one `foreshadow` spinal railgun at `(0, 2)` with `R0`, occupying
`(0,2)..(0,5)` and pointing along the ship's nose; the fixed weapon is aimed by the
ship-level `AimAt` behavior. Its AI opens with `Attack(target: NearestEnemy)`, kites at
4500 while an enemy is within 6000, and flees only below 20% health. `Blackbird` is a
long slender dart with a nose spike, two horizontal `fin_1x2` wings, a single laser and
4 thrusters; its AI opens with `Chase(target: NearestEnemy)`, kites at 2500 within 4000,
and flees below 40% health. The bomber therefore has greater hull, weapon count,
weapon reach and stored power (1200 vs 1100) but lower thrust (600 vs 1000) and turn
rate (260 vs 460) than the scout.

Every triangle sits on the silhouette boundary with its empty side facing outward,
so no block touches a triangle's empty edge. The new starters fit within
`x=-9..9, y=-2..5` (`B2`) and `x=-3..3, y=-3..5` (`Blackbird`); generation covers
consumer demand on both (60/6 and 35/25).
AI profiles, design names, ids, and schema version are defined by the starter
documents and are independent of the block layout.

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

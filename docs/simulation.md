# Simulation Spec — Ships and Blocks

## Canonical types

- `BlockPlacement` — static block description (`id`, `position`, `block_type`, `rotation`).
- `GridBlock` — one live block in a grid (`placement` + `entity`).
- `BlockRef` / `BlockOwner` — identity and ownership components on block entities.
- `ShipGrid { blocks: HashMap<Uuid, GridBlock>, cells: HashMap<IVec2, Uuid> }` —
  authoritative placement and identity. `cells` is derived via `rebuild_from_layout`
  and `refresh_cells`.
- `ShipStats` — single aggregate per ship (mass, health, ranges, power, cargo,
  thrust). Derived by `BlockRegistry::stats_from_grid` on every `ShipChanged`.

The old `BlockData`, `Block`, `BlockState`, `Blocks`, `ShipData`, `ShipState`,
`ShipCalculated*`, and `ComputedShipStats` types are deleted.

## Blocks

Grid cell = `GRID_SIZE = 64.0` px. A block occupies the cells of its `BlockShape`;
placement validity in the editor requires face-match connectivity. Multi-cell
blocks use a filled voxel rectangle (`core` is 3×3, `spectre` 3×3, `foreshadow` 3×9).

`BlockDef` fields: `name`, `block_type`, `block_group`, `mass`, `health`,
`shape`, `cost`, `features: Vec<Feature>`, `drops`. `BlockRegistry` (resource)
maps string ids to defs, rejects duplicates, and keeps an `iter()` order sorted
once at registration by block group.

Features (`FeatureConfig`/`Feature`, shared between RON config and runtime):
`Texture`, `Turret`, `Generator`, `Battery`, `Consumer`, `Booster`, `Thruster`,
`Storage` (capacity only). Power components are shared between serde and runtime.

Block entities carry `BlockRef`, `BlockOwner`, `Health`, `Team`, a sensor
`Collider` in a per-team hitbox layer, and all feature components. The only
child is the rotating turret barrel (`TurretSprite`); extra textures are
render-only children.

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

`handle_block_destroy` writes `ShipMutation::DestroyBlock` for blocks at
`health <= 0`. `apply_ship_mutations` (FixedPostUpdate, `Death` set) batches per
ship: remove blocks, delete detached fragments, kill the ship on core loss,
rebuild stats/bounds/collider from `ShipGrid.cells`, emit drops, and send
`ShipChanged`. Colliders are rebuilt from `grid.cells` only.

## Ships

`ShipDesign::spawn` builds a `ShipGrid` from a layout, derives `ShipBuildResult`,
spawns the ship with `RigidBody::Dynamic`, voxel collider, `ShipStats`,
`ShipGrid`, `AiProfile`, `AiRuntime`, `DespawnOnExit(SceneState::Game)`, and
spawns each block entity with its identity. The debug `ShipUIRoot` shows
behavior, stats, and pooled cargo.

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

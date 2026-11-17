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
`Texture`, `Turret`, `EnergyConsumer`, `PowerProducer`, `Thruster`,
`EfficiencyProvider`, `Storage` (capacity only). Producers/consumers are single
serde + component types; there is no separate power config struct.

Block entities carry `BlockRef`, `BlockOwner`, `Health`, `Team`, a sensor
`Collider` in a per-team hitbox layer, and all feature components. The only
child is the rotating turret barrel (`TurretSprite`); extra textures are
render-only children.

## Power

`PowerProducer { output }` creates power from nothing. `EnergyConsumer { rate }`
consumes it. `update_power_efficiency` (FixedFirst) computes per ship:

```text
production = sum(producer.output)
consumers  = consumers with rate > 0
share      = production / consumers.len()
efficiency_i = 1                       if no consumers or production >= demand
             = clamp(share / rate_i,0,1) otherwise
```

Only consumers get `PowerEfficiency`. `ShipStats` exposes `power_production`,
`power_demand`, `power_used`, and `efficiency`. A powered turret that is not
fully supplied cannot fire; unpowered turrets are unaffected.

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

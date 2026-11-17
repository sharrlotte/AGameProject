# Economy — Items, Storage, Ammo

## Items

`ItemRegistry` (resource, `HashMap<ItemType, ItemDef>`) holds item definitions.
Registered in `PreStartup` — exactly one item exists:

| ItemType | Name | Group | Rarity | `occupied` |
|---|---|---|---|---|
| `Steel` | `"steel"` | `Resource` | `Common` | `1` |

`Rarity::color()` maps rarity tiers to HDR colors (bloom-friendly, values > 1) used for
drop sprites; storage UI and drops fall back to white when unregistered.

## Drops

- Source: block destruction writes `DropEvent` for **non-player** teams.
- Sink: `handle_drop` (`FixedPostUpdate`) spawns a `Lifetime::seconds(drop_despawn_in_seconds)`
  pickup sprite (`300.0` via `game.ron`), z = `LAYER_DROP = 110`.
- Collection: `collect_drop` (`FixedUpdate`, `SceneState::Game` only) — player storages
  within `drop_collect_radius` (`1000.0` via `game.ron`) pull drops in; consumed at
  `DROP_PICKUP_RADIUS = 32.0` px. Decrements inventory into any non-full storage
  on the player ship.

## Storage

Storage exists only as `Feature::Storage(capacity)` on blocks (currently only the Core via
`Storage((capacity: 1000))`). It is a pure capacity contributor: `stats_from_grid`
accumulates it into `ShipStats::cargo_capacity`. There is no per-block inventory
component — collected items live on `ShipStats` (`cargo_capacity`, `cargo_used`,
`cargo_items` keyed by item id).

Collection (`collect_drop`, see Drops) consumes a drop atomically: the full amount must
fit or the drop stays in the world. `rebuild_cargo_capacity` (`ShipChanged`) recomputes
pool size; losing storage discards the excess. `ShipStats::is_full` is
`cargo_used >= cargo_capacity`.

## Ammo

`Ammo` (`capacity`, `crafting: HashMap<ItemType, u32>`, `icon`) + `AmmoState (current)`.
Inserted via `AmmoBuilder` at block spawn; ships with ammo turrets currently spawn with
full `AmmoState` (`current = capacity`).

- Consumption: one ammo per barrel per shot (`Single` / `Burst`); ammo is drawn only when
  `take()` succeeds.
- Player-facing behavior: `update_ammo` (`FixedUpdate`) spawns a `NoAmmoWarningUI`
  (warning-triangle mesh + exclamation meshes via mesh shapes) above the ammo-less block;
  `update_no_ammo_warning` follows the block's `GlobalTransform` and removes the marker
  when refilled.
- Crafting inputs: Swarmer (3 Steel per missile), Spectre (1 Steel per bullet),
  Colaris (1 Steel per bullet).

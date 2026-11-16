# Economy — Items, Storage, Ammo

## Items

`ItemRegistry` (resource, `HashMap<ItemType, ItemDef>`) holds item definitions.
Registered in `PreStartup` — exactly one item exists:

| ItemType | Name | Group | Rarity | `occupied` |
|---|---|---|---|---|
| `Steel` | `"steal"` (sic) | `Resource` | `Common` | `1` |

`Rarity::color()` maps rarity tiers to HDR colors (bloom-friendly, values > 1) used for
drop sprites; storage UI and drops fall back to white when unregistered.

## Drops

- Source: block destruction writes `DropEvent` for **non-player** teams.
- Sink: `handle_drop` (`FixedPostUpdate`) spawns a `Lifetime::seconds(300)` pickup sprite,
  z = `LAYER_DROP = 110`.
- Collection: `collect_drop` (`FixedUpdate`, `SceneState::Game` only) — player storages
  within `DROP_COLLECT_RADIUS = 1000` pull drops in; consumed at `sqrt(1000) ≈ 31.6` px
  (implementation detail — see defects.md). Decrements inventory into any non-full storage
  on the player ship.

## Storage

`Storage` (`capacity`) on blocks (currently only the Core via feature) + `StorageState`:

| Method | Semantics |
|---|---|
| `add(item_type, count)` | True if `current + count <= capacity`; otherwise false (drop not lost — stays in world) |
| `take(item_type, count)` | Fails if not enough of that item; decrements totals |
| `get` / `iter` | Read-only access |
| `is_full()` | Percent-based; **panics** if `current > capacity` |

Storage state is a flat `HashMap<ItemType, u32>` with a running `current` total
(not capacity-checked against `items` entries; invariant `current == sum(items)`).

`ComputedShipStats.storages` aggregates storage entities per ship; `items` aggregates
item totals; `is_full` is tracked per ship (`true` unless at least one attached storage
is not full).

## Ammo

`Ammo` (`capacity`, `crafting: HashMap<ItemType, u32>`, `icon`) + `AmmoState (current)`.
Inserted via `AmmoBuilder` at block spawn; ships with ammo turrets currently spawn with
full `AmmoState` (`current = capacity`).

- Consumption: one ammo per barrel per shot (`Single` / `Burst`).
- Replenishment: `AmmoState::refill()` computes how much can be made from storage contents
  (`min over crafting requirements`) and consumes the items. **Never called by any system.**
- Player-facing behavior: `update_ammo` (`FixedUpdate`) spawns a `NoAmmoWarningUI`
  (warning-triangle mesh + exclamation meshes via `bevy_hanabi`-free mesh shapes) above the
  ammo-less block; `update_no_ammo_warning` follows the block's `GlobalTransform` and
  removes the marker when refilled.
- Crafting inputs: Swarmer (3 Steel per missile), Spectre (1 Steel per bullet),
  Colaris (1 Steel per bullet).

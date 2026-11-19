## Context

In `SceneState::Game`, ship inspection currently lives inside the dev-only `ShipDebugPlugin` (`core/src/content/ship/debug/`), which maintains `ShipDebugSelection` (an `EntityHashSet`), spawns a world-space `Text2d` (`ShipUIRoot`) child on every ship, and draws debug gizmos. Meanwhile, production builds have `ShipList` (left-bar colored-square buttons that cycle `CameraTarget` / `ControlledShip`) and `SelectablePlugin` (`Ctrl+drag` fleet box-selection), with no way to inspect a ship's hull blocks or live stats.

## Goals / Non-Goals

**Goals:**
- Promote ship selection and inspection to a production feature in `SceneState::Game` using a dedicated `InspectedShip(Option<Entity>)` resource.
- Gate enemy ship selection (`!team.is_player()`), debug gizmos, and raw AI/ammo debug fields behind `#[cfg(feature = "dev")]`.
- Display a bottom-right HUD Ship Inspector panel showing the ship's block diagram rendered with actual block sprite `ImageNode`s (respecting rotation/flip and live block HP tinting) and near-real-time ship stats.
- Sync `ShipList` button clicks to also set `InspectedShip`.
- Avoid per-tick UI entity respawn/despawn churn by building the block diagram only when `InspectedShip` changes or `ShipChanged` fires, and mutating HP tints and stat text/bars in-place on a short timer (~80ms).

**Non-Goals:**
- Multi-ship tabbed inspection or fleet-wide aggregate stats in the inspector panel (single-ship `InspectedShip` only).
- Interactive block clicking/editing inside the inspector diagram.
- Changing `Ctrl+drag` (`SelectablePlugin` / `EntitySelectEvent`) or `CameraTarget` / `ControlledShip` semantics for in-world clicks.

## Decisions

### 1. Selection State Ownership & Module Layout
- **Decision**: Move ship selection state (`InspectedShip`, `ShipSelected`, `ShipSelectionCleared`, `on_ship_click`, `apply_selection`, `clear_selection_on_escape`, `remove_despawned_ships`) into a production `core/src/content/ship/inspect/` (or `select/`) module with `mod.rs`, `component.rs`, `system.rs`, `lib.rs`.
- **Rationale**: Keeps selection state in `content/ship` alongside `Ship` and `ShipStats`, so both `scene/game` (the HUD inspector and `ShipList`) and `content/ship/debug` (`#[cfg(feature = "dev")]` gizmos) can read `InspectedShip` without circular dependencies.
- **Alternative considered**: Putting `InspectedShip` in `scene/game/`; rejected because `on_ship_click` is attached on ship spawn in `content/ship/component.rs` and `ShipDebugPlugin` lives in `content/ship/debug/`.

### 2. Enemy Gating & Dev-Only Debug Integration
- **Decision**: Observe `on_ship_click` on all spawned ships in all builds. Inside `on_ship_click` (and `apply_selection`), reject ships where `!team.is_player()` when `#[cfg(not(feature = "dev"))]`. Remove `ShipDebugSelection` and `ShipUIRoot` (`Text2d`); have `draw_ship_debug_gizmos` read `Res<InspectedShip>`.
- **Rationale**: Single source of truth for which ship is inspected, no duplicate click handlers, and zero dev-only `Text2d` entities spawned per ship.
- **Alternative considered**: Keeping `ShipDebugSelection` separate in `dev`; rejected to avoid conflicting click observers and duplicated selection state.

### 3. Bottom-Right HUD Inspector & In-Place Real-Time Updates
- **Decision**: Create `core/src/scene/game/inspector/` (`mod.rs`, `component.rs`, `system.rs`) spawned as a child of the game UI root (`DespawnOnExit(SceneState::Game)`), anchored bottom-right (`Visibility::Hidden` / `Display::None` when `InspectedShip` is `None`).
  - **Structural rebuild**: Triggered only when `inspected.is_changed()` or `ShipChanged` matches `inspected.0`. Despawns and rebuilds the block diagram `ImageNode`s using `BlockRegistry` (`Feature::Texture` and `Feature::Turret` handles, `placement.xform` rotation/flip, and `InspectorBlockNode(block_entity)`).
  - **Live refresh (~80ms timer)**: Updates `InspectorBlockNode` `ImageNode.color` from `Health` (`health / max_health`), updates stat `Text` and progress bar widths from `ShipStats`, `Readiness`, `Brain`, `Doctrine`, `Capability`, and (under `#[cfg(feature = "dev")]`) `Skill`, `Steering`, and block `Ammo`/`AmmoState` summaries.
- **Rationale**: Rebuilding UI nodes every 80ms would cause layout churn; separating structural rebuild from in-place mutation keeps frame times flat while feeling real-time.

### 4. `ShipList` Integration
- **Decision**: In `on_ship_button` (`core/src/scene/game/mod.rs`), clicking a player ship button in `ShipList` also writes `ShipSelected { entity }` (or sets `InspectedShip(Some(entity))` when not already deselected) alongside the existing Follow/Pilot cycle.
- **Rationale**: Gives players two natural ways to open the inspector—clicking a ship in space or clicking its entry in the fleet list.

## Risks / Trade-offs

- **Large ships with many blocks**: Spawning one `ImageNode` per block (and turret overlay) on a 100+ block ship happens once per ship selection or `ShipChanged` event rather than every tick. Scaling block cell size dynamically to fit a fixed max diagram box (e.g. `160x160` px) prevents huge ships from overflowing the HUD.
- **Rotated / flipped block sprites in UI**: Bevy UI `ImageNode` supports `flip_x` / `flip_y`, and `UiTransform` (or `Transform` rotation) handles 90-degree increments (`Rot90`). We must verify exact `Rot90` sign convention against `core/src/content/block/feature/sprite/` so UI block orientations match in-world sprites.
- **Turret blocks with separate base + turret textures**: Following `core/src/scene/editor/ui.rs`, spawning child `ImageNode`s for both `Feature::Texture` and `Feature::Turret` (when present on a block) ensures turrets render with their hull base underneath.

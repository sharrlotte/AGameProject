# UI Spec — HUD and Panels

All HUD elements are UI nodes (not world-space canvases) except damage numbers (`Text2d` at `LAYER_DAMAGE_TEXT = 400`).

## Game scene HUD

- **Top bar** — `FPS: <value>, Vec2(<x>, <y>)` — `FrameTimeDiagnosticsPlugin::FPS`
  (smoothed), prefixed with `TextFont { font_size: 10.0 }`.
- **Left — ship list** — refreshed every 200 ms (`on_timer(200ms)`) but updated
  incrementally: button entities are keyed by their target ship and survive a refresh.
  Mini-map icon nodes are non-pickable so the button is the sole click target. One button
  per player ship, showing:
  - a 6 px-ish scaled `(ship_size / GRID_SIZE + 2) × SMALL_GRID_SIZE = 6.0` px box,
  - absolute-positioned mini block icons colored by health fraction (see scenes.md),
  - white border by default; green border = controlled, cyan = followed.
  - Click: first click → `following`, click again → `controlling`, third click → clear both.
- **Bottom bar** — `SelectablePanel`: after any drag-select, one `"<TYPE>: <len>"` label per
  selection group. Stale between events.
- **Selection outline** — drag rectangle drawn via gizmos (`SelectablePlugin`). Both the
  dragging rectangle and entourage highlights use world-space gizmos.
- **Ship debug gizmos** — with the `dev` feature, `ShipDebugPlugin` draws AI behavior
  geometry, hull/bounds with a core marker, heading/velocity/force, and turret aim rays
  only for the ship in `InspectedShip`.
- **Ship click picking** — block visual sprites receive `Pickable` so clicks bubble to
  the ship root, and non-interactive HUD nodes are `Pickable::IGNORE` so they do not
  swallow world clicks. Interactive buttons keep their picking behavior. Non-player ships
  can be clicked only when the `dev` feature is enabled **and** they are `Visibility::Visible`
  (detected by player `TeamIntel.contacts`). Undetected enemies cannot be selected or
  inspected.

## Tactical minimap

`MinimapPlugin` (`scene/game/minimap/`) spawns an always-on top-right `MinimapRoot`
(`DespawnOnExit(SceneState::Game)`, independent of the debug-only top bar). The widget is a
single-border radar-green circular clipped UI node sized and offset by `config/base/minimap.ron`;
it is north-up (never rotates) and has no inner ring, sweep hand, or center arrow.

- **Center & scale**: Always centered on the active `Camera2d` world position in real time.
  The world radius scales dynamically as `active_radar_range * radar_scale_factor` (`1.2`),
  resolving `active_radar_range` via `ControlledShip.controlling` → `CameraTarget.following`
  → maximum `Capability.max_radar_range` among alive player ships → `fallback_world_radius`.
- **Real-time markers & glow**: `update_minimap` updates allied player-team ships and live
  `TeamIntel.contacts` in real time (omitting `alerts` and `last_known` so each enemy renders
  as one dot), with a `BoxShadow` glow on visible dots.
- **Lost-contact flash**: When an enemy leaves `TeamIntel.contacts`, it blinks
  `lost_flash_count` times over `lost_flash_secs` at its world-anchored last position before
  hiding.

## Ship Inspector HUD

`ShipInspectorRoot` (`scene/game/inspector/`) is anchored at the bottom-right of `SceneState::Game` (`Visibility::Hidden` / `Display::None` when `InspectedShip` is `None`).

- **Block diagram** (`InspectorBlockGrid`): rebuilt only when `InspectedShip` changes or `ShipChanged` fires for the inspected ship. Renders each block's `Feature::Texture` and `Feature::Turret` `ImageNode`s scaled to fit `140×140` px and oriented with `placement.rotation` and `def.shape.xform()`.
- **Live refresh** (`on_timer(80ms)`): mutates `InspectorBlockNode` `ImageNode.color` from block `Health` (`> 0.75` white, `0.4..=0.75` amber, `< 0.4` red) and updates Hull HP, Power (`production`/`demand`/`supply`), Battery (`stored`/`capacity`), Ammo readiness, Cargo, Mobility (`mass`/`thrust`/`turn`), Combat (`range`/`radar`), and AI (`Role`/`Doctrine`/`Brain.current`) in-place.
- **Dev section** (`#[cfg(feature = "dev")]`): appends `Skill`, `Steering`, and per-block `AmmoState` totals.

## Damage / warning icons

- Damage floating text: red `Text2d`, rendered through the `DamageTextPool` (64 slots),
  lifetime `damage_number_lifetime` (`game.ron`, 1.5 s).
- No-ammo warning: 3-mesh triangle + exclamation (border triangle black, inner triangle
  yellow, exclamation black) hovering above the block, `LAYER_BLOCK_ICON + offsets`,
  removed when ammo > 0.

## Shared button widget

`scene/widget.rs` provides the single button builder. All interactive UI buttons are
spawned through it. The builder inserts the button's action component (defined in the
owning domain module), `ButtonVisuals`, `ButtonVisualState` and `ButtonInteraction`, and
attaches the domain-provided `PointerClick` observer plus shared `PointerOver` /
`PointerOut` / `PointerPress` / `PointerRelease` observers. Pointer events bubble from
text sections to the button, so clicks and hover feedback work over both the label and
blank space. Domain systems set `ButtonVisualState` (`Normal` / `Selected` / `Disabled`);
one generic system derives the final color from `ButtonVisuals` and that state.

## Reusable scroll widget

`scene/widget/scroll/` provides `ScrollPlugin` and `spawn_scroll_area`. A scroll area is a
`ScrollWrapper` (caller-supplied `Display` and tab marker) containing a `ScrollViewport`
(`Overflow::scroll_y()`, `flex_grow: 1.0`, `flex_basis: 0`, `min_height: 0`, `row_gap` supplied
by the caller) and a sibling `ScrollbarTrack` holding an absolutely positioned
`ScrollbarThumb`. Each scroll area occupies only the remaining space in a column panel, and
the builder returns the inner viewport entity so callers can attach a content marker.

`send_scroll_events` translates `MouseWheel` input on hovered UI entities into bubbling `Scroll`
events. `on_scroll_handler` applies a scroll event to the nearest scrollable ancestor, consumes
only the delta actually applied, and clamps `ScrollPosition` to `0.0..=max_offset` per axis.
`sync_scrollbars` hides the track when `ComputedNode.content_size().y <= size().y + 1.0`, and
otherwise sizes the thumb to `clamp(view / content * 100, 10, 100)%` and sets its top offset
from the scroll ratio over the remaining track space. The thumb is a visual indicator;
drag-to-scroll is not part of the widget.

## Ship editor left panel

The left panel has a fixed "Back to Home" button, a tab bar, and three mutually exclusive tabs
(`Blocks`, `Ships`, `AI`) built with `spawn_scroll_area`. The tab marker and `Display` live on
the wrapper, so switching tabs hides/shows the list and its scrollbar together: `sync_tabs`
sets `Display::Flex` on the active wrapper and `Display::None` on the others.

Fixed-height list items (`BlockSelectButton`, ship rows, `AiPresetButton`) use
`flex_shrink: 0.0` so long lists overflow the viewport instead of being compressed. Ship rows
are cleared and repopulated in the inner `ShipListContent` viewport, leaving the sibling
scrollbar intact across refreshes.

## Ship editor deployed fleet

The editor's right panel lists the deployed fleet in a scroll area (`FleetListPanel` with
an inner `FleetListContent` viewport). Each row is a `RemoveFleetButton` labelled with the
design name; removing it changes `DeployPayload.fleet` only and rewrites `<data_dir>/fleet.ron`.
The list is rebuilt when the deploy payload changes and on editor entry.

The Deploy button upserts the current grid design into the fleet by name. It is disabled
only while the current grid matches the fleet entry of that name — comparing name,
doctrine, and block position/type/rotation while ignoring `BlockPlacement.id`. An edited
grid therefore re-enables Deploy so the fleet entry can be updated.

## Setting panel

See scenes.md (Volume / Bloom / Close).

## FPS / diagnostics

`FrameTimeDiagnosticsPlugin(100)`, `EntityCountDiagnosticsPlugin(100)` registered in main.
Debug builds additionally register `WorldInspectorPlugin` (F12). With feature `dev`,
`RemotePlugin`/`BrpExtrasPlugin` and `ShipDebugPlugin` are registered; the latter owns
ship click selection, the scoped debug label, and the ship debug gizmos.

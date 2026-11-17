# UI Spec — HUD and Panels

All HUD elements are UI nodes (not world-space canvases) except the per-ship debug label
(`ShipUIRoot`, `Text2d` in-world) and damage numbers (`Text2d` at z = 1,000,000).

## Game scene HUD

- **Top bar** — `FPS: <value>, Vec2(<x>, <y>)` — `FrameTimeDiagnosticsPlugin::FPS`
  (smoothed), prefixed with `TextFont { font_size: 10.0 }`.
- **Left — ship list** — rebuilt every 200 ms (`on_timer(200ms)`). One button per player
  ship, showing:
  - a 6 px-ish scaled `(ship_size / GRID_SIZE + 2) × SMALL_GRID_SIZE = 6.0` px box,
  - absolute-positioned mini block icons colored by health fraction (see scenes.md),
  - white border by default; green border = controlled, cyan = followed.
  - Click: first click → `following`, click again → `controlling`, third click → clear both.
- **Bottom bar** — `SelectablePanel`: after any drag-select, one `"<TYPE>: <len>"` label per
  selection group. Stale between events.
- **Selection outline** — drag rectangle drawn via gizmos (`SelectablePlugin`). Both the
  dragging rectangle and entourage highlights use world-space gizmos.

## Per-ship in-world label

`ShipUIRoot` child of the ship; contains a `Text2d` rebuilt every fixed tick:

```
Ship: <Entity id>
C: <AiBehavior label>
B: <ShipBehavior or None>
CARGO <used> / <capacity> {item breakdown}
S: <ShipStats debug>
<ammo debug dump for each ammo-capable child>
```

Reverse-rotated to face screen upright (`global_transform.rotation().inverse()`),
z = `LAYER_SHIP_UI = 300`. This is a development visual leaking into release builds.

## Damage / warning icons

- Damage floating text: red `Text2d`, lifetime 1.5 s.
- No-ammo warning: 3-mesh triangle + exclamation (border triangle black, inner triangle
  yellow, exclamation black) hovering above the block, `LAYER_BLOCK_ICON + offsets`,
  removed when ammo > 0.

## Setting panel

See scenes.md (Volume / Bloom / Close).

## FPS / diagnostics

`FrameTimeDiagnosticsPlugin(100)`, `EntityCountDiagnosticsPlugin(100)` registered in main.
Debug builds additionally register `WorldInspectorPlugin` (F12) and, with feature `dev`,
`RemotePlugin`/`BrpExtrasPlugin`.

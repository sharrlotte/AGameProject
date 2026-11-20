# Scenes Spec

## Home

Entry scene (`SceneState::Home`). UI column: **Play**, **Quick Battle**, **Ship Editor**,
**Settings**, **Exit**.
- **Play** opens the Save Slot Picker (`SlotPickerState::Open`); see ui.md. `New World` emits
  `CreateSlotEvent`, `Load` emits `LoadSlotEvent`, and `Delete` removes the slot's `.db` files
  after confirmation.
- **Quick Battle** emits `StartQuickBattleEvent` and enters `SceneState::Game` on the
  `InMemoryWorldStore` sandbox without reading or writing save slots.
- **Ship Editor** sets `EditorReturnTarget::Home`. **Settings** opens `SettingState::Open`.
  **Exit** writes `AppExit::Success`.
No game entities exist here.

## Game

`SceneState::Game`. Content:

- HUD root (`DespawnOnExit(SceneState::Game)`): top bar (debug-only FPS + camera coords), middle row
  (ship list — left scroll panel rebuilt every 200 ms, re-showing *player* ships with
  per-block health color: green > 0.75, yellow > 0.5, red ≤ 0.5), bottom bar (selection
  echo panel, `SelectablePanel`).
- Background parallax starfields: far/mid/near colored sprites wrapped around the camera,
  tagged `DespawnOnExit(SceneState::Game)` (despawn on scene exit).
- Battle-core (see mission.md): mission loads async; ship spawns driven by the mission
  def; outcome is win/lose; overlay on `Ended`. Top-up spawner deleted; `check_ship`
  gone.
- Ship mutations rebuild ship stats and bounds from `BlockGrid` and the compound collider
  from the baked hitboxes of live blocks, then emit `ShipChanged`; `update_stats`
  (`ShipChanged`, `OnMessage`) recomputes `ShipStats`. Detached components become
  `Debris` bodies with their own grid, collider, lifetime, and drops.
- Bottom-right `ShipInspectorRoot` (`scene/game/inspector/`): hidden when `InspectedShip` is `None`; displays the ship's block sprite diagram (rebuilt on `InspectedShip` change or `ShipChanged`, tinted in-place by block `Health`) and near-real-time stats (`ShipStats`, `Readiness`, `Brain`, `Doctrine`, `Capability`, plus `Skill`/`Steering`/block `Ammo` in `dev`).

Gameplay selection requires `Ctrl` + left-drag and remains player-only; `Home`/`Game` do not scroll-select. Left-clicking a player ship in-world or clicking a ship in `ShipList` sets `InspectedShip` (left-clicking enemy ships is gated to `dev`), and `Escape` clears `InspectedShip`.

## Ship Editor

`SceneState::ShipEditor`. Purpose: assemble player ship designs and persist them as
schematics.

Layout: left panel (220 px wide) — Back button, tab strip (**Blocks** / **Ships** / **AI**),
block palette (scrollable, one button per registered block type, with block icon layers +
name), ship-list (saved schematics, click to load), or AI **Doctrine** selector buttons (`Auto`,
`Brawler`, `Skirmisher`, `Sniper`, `Support`, `Escort`); center grid area; right panel
(250 px) — selected block properties (Name/Cost/Mass/HP), instructions, **Deploy** button
(stores the assembled ship as `DeployPayload.current_ship` — consumed by the next mission
spawn as the player's ship), **Start Game**.

Mechanics:

- Virtual grid, z = editor grid (`GRID_SIZE`), camera pans via left-mouse drag
  (`pan_speed = ortho.scale * 0.5`), zoom clamps `[0.1, 5.0]`
  (inconsistent with the game's `[0.2, 30.0]`).
- Placement requires a shared orthogonal edge (Manhattan distance 1) with an existing
  block (`check_placement_validity`), using the pending rotated footprint; diagonal
  contact does not connect. An empty grid accepts anything.
- **Q / E** rotate the pending block counter-clockwise / clockwise (bindings from
  `setting.ron` → `keys`). The rotation applies to the anchor (rotated footprint size /
  2), the preview, validity checks, and the placed `BlockPlacement.rotation`.
- The placement preview is a single persistent entity that updates in place only when the
  hovered cell, selected block, rotation, or grid changes; children are rebuilt only when
  the selected block changes.
- **Right-click deletes** blocks at cursor.
- Save triggers: any block add/remove writes `ShipSaveChanged` (`handle_ship_saving`),
  which requires ≥ 2 blocks and exactly one Core (`BlockRegistry::validate_ship`); it emits
  `DesignSaveRequested`, persisted to the active slot's `designs` table (`ActiveWorldDb`) or
  to `<data_dir>/sandbox_designs/` (`HomeSandbox`), and emits `ShipSavesChanged` to refresh
  the ship list UI.
- Blocks listed in the editor are `Block` entities spawned without physics; they are
  tagged `DespawnOnExit(SceneState::ShipEditor)`.
- Editor does not place `Ammo`/`Storage`/power/`Thruster` feature children — only
  `Texture` and `Turret` sprite layers are reproduced; everything else is dropped.

Ship list: `refresh_ship_list` reads the in-memory `ShipCatalog` (hydrated from the slot or
sandbox on session change) and rebuilds the UI on `ShipSavesChanged` (and `OnEnter(ShipEditor)`).

Deletion nuance: `handle_block_deletion` deletes the block whose grid-anchor matches the
cursor; `remove_block_at` removes all voxels of that block.

## Pause

`SceneState::Game` or `ShipEditor`, toggled by `Escape`, mapped to
`PauseState::{Playing, Paused}` (`OnEnter(Paused)` pauses `Time<Virtual>`, `OnExit`
unpauses). Menu: **Resume**, **Settings**, **Ship Editor** (only in a `WorldSession::SaveSlot`
session, setting `EditorReturnTarget::Game`), **Back to Home**.
In `SceneState::Game` with `WorldSession::SaveSlot`, **Back to Home** triggers `SaveWorldEvent`
(save + WAL checkpoint) and defers the scene transition via `PendingExitAction::Home`.
Application exit is available only from **Home**.

## Settings

Global settings UI (`SettingState::Open`), available from Home and Pause; modal absolute
panel (opacity 0.8). Controls:

- Volume — ±10 % linear (`GlobalVolume`); persisted to `config/setting.ron` by
  `save_setting_on_change` (0.5 s debounce after the change settles).
- Bloom — toggles `Bloom.intensity` between `0.3` (default) and `0.0` (OFF); also
  persisted to `config/setting.ron`.
- AI Skill — `[-]` / `[+]` stepper clamping `GameSetting.ai_level` within `1..=10`;
  persisted to `config/setting.ron`.
- Close.

`volume` and `bloom` are written to `config/setting.ron` and hot-reloaded on edit (dev
builds with the `file_watcher` feature; production applies settings via the UI or on
restart). All other state is in-memory only.

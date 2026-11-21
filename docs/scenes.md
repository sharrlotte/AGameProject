# Scenes Spec

## Home

Entry scene (`SceneState::Home`). UI column: **Play**, **Quick Battle**, **Worlds**, **Ship Editor**,
**Settings**, **Exit**.
- **Play** emits `ContinueWorldEvent` to continue the active or most recent save slot (creating a new slot when none exist).
- **Worlds** pushes `Modal::SlotPicker` (see ui.md). `New World` emits
  `CreateSlotEvent`, `Load` emits `LoadSlotEvent`, and `Delete` removes the slot's `.db` files
  after confirmation.
- **Quick Battle** emits `StartQuickBattleEvent` and enters `SceneState::Game` on the
  `InMemoryWorldStore` sandbox without reading or writing save slots.
- **Ship Editor** sets `EditorReturnTarget::Home`. **Settings** pushes `Modal::Settings`.
  **Exit** writes `AppExit::Success`.
No game entities exist here.

## Game

`SceneState::Game`. Content:

- HUD root (`DespawnOnExit(SceneState::Game)`): top bar (debug-only FPS + camera coords and `CargoHudText`), middle row
  (ship list — left scroll panel rebuilt every 200 ms, re-showing *player* ships with
  per-block health color: green > 0.75, yellow > 0.5, red ≤ 0.5), bottom bar (selection
  echo panel, `SelectablePanel`), and `MinimapPlugin` radar.
- Background parallax starfields: far/mid/near colored sprites wrapped around the camera,
  tagged `DespawnOnExit(SceneState::Game)` (despawn on scene exit).
- Quest system (see quests.md): authored tutorial quests auto-activate from prerequisites;
  generated quests appear only when a sector is discovered. No win/lose overlay; destroying
  the last enemy does not end play. Last-player-ship destruction respawns a starter ship at
  the origin.
- Skirmish spawner (`scene/game/skirmish/`): `config/base/skirmish.ron` supplies the enemy spawn
  list for the unsaved QuickBattle sandbox (Blackbird, Raider, Interceptor, Aegis, Vanguard); a
  snapshot load takes over Game entry instead.
- Ship mutations rebuild ship stats and bounds from `BlockGrid` and the compound collider
  from the baked hitboxes of live blocks, then emit `ShipChanged`; `rebuild_cargo_capacity`
  (`on_message::<ShipChanged>`) recomputes cargo capacity while `update_stats` sums thrust/rotate in `FixedPreUpdate`. Detached components become
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
(upserts the assembled ship into `DeployPayload.fleet` — also shown in the Deployed Fleet panel), **Start Game** (emits `ContinueWorldEvent`).

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
- Blocks placed in the editor are `BlockRef` entities spawned without physics under `ShipBase`
  (`DespawnOnExit(SceneState::ShipEditor)`).
- Editor reproduces `Texture`, `Turret` sprite layers, and `Booster` overdrive fields; other
  gameplay features are not instantiated in the editor.

Ship list: `refresh_ship_list` reads the in-memory `ShipCatalog` (hydrated from the slot or
sandbox on session change) and rebuilds the UI on `ShipSavesChanged` (and `OnEnter(ShipEditor)`).

Deletion nuance: `handle_block_deletion` deletes the block whose grid-anchor matches the
cursor; `remove_block_at` removes all voxels of that block.

## Pause

`SceneState::Game` or `ShipEditor`, toggled by `Escape` via `ModalStack` (`Modal::Pause`, which pauses
`Time<Virtual>` while on top). Menu: **Resume**, **Settings**, **Ship Editor** (only when `WorldContext::is_save_slot()`
is true, emitting `SaveWorldEvent` and setting `EditorReturnTarget::Game`), **Back to Home**.
In `SceneState::Game` with `WorldContext::is_save_slot()`, **Back to Home** triggers `SaveWorldEvent`
(save + WAL checkpoint) and defers the scene transition via `PendingExitAction::Home`.
Application exit is available only from **Home**.

## Settings

Global settings UI (`Modal::Settings`), available from Home and Pause; modal absolute
panel (opacity 0.99). Controls:

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

## Modal Ownership

Pause, Settings, and the Save Slot Picker SHALL be owned by a single `ModalStack` resource with variants `Modal::{Pause, Settings, SlotPicker}`. `PauseState`, `SettingState`, and `SlotPickerState` SHALL NOT exist. Escape SHALL pop the top modal, or push Pause when the stack is empty in `Game`/`ShipEditor`. Leaving `SceneState::Game` SHALL clear the stack and unpause `Time<Virtual>`. Virtual time SHALL be paused only while Pause is the top modal during gameplay. Modal UI roots SHALL use `ModalRoot(Modal)` and despawn when their modal closes.

# Scenes Spec

## Home

Entry scene (`SceneState::Home`). UI column: **Play** (or **Continue** when `saves/battle.ron`
exists), **Quick Battle**, **Ship Editor**, **Settings**, **Exit**.
- **Play / Continue** sets `BattleMode::Campaign`; **Continue** emits `LoadWorldEvent`, while
  **Play** enters `SceneState::Game` fresh.
- **Quick Battle** sets `BattleMode::QuickBattle` and enters `SceneState::Game` without
  reading or writing campaign saves.
- **Settings** opens `SettingState::Open`. **Exit** writes `AppExit::Success`.
No game entities exist here.

## Game

`SceneState::Game`. Content:

- HUD root (`SceneEntity(Game)`): top bar (debug-only FPS + camera coords), middle row
  (ship list — left scroll panel rebuilt every 200 ms, re-showing *player* ships with
  per-block health color: green > 0.75, yellow > 0.5, red ≤ 0.5), bottom bar (selection
  echo panel, `SelectablePanel`).
- Background parallax starfields: far/mid/near colored sprites wrapped around the camera,
  tagged `SceneEntity(Game)` (despawn on scene exit).
- Battle-core (see mission.md): mission loads async; ship spawns driven by the mission
  def; outcome is win/lose; overlay on `Ended`. Top-up spawner deleted; `check_ship`
  gone.
- `check_rebuild_ship_collider` (ship) watches `BlockDestroyedEvent`s.
- `ShipUIRoot` child of each ship (debug builds only): `Text2d` debug dump.

Selection requires `Ctrl` + left-drag; `Home`/`Game` do not scroll-select.

## Ship Editor

`SceneState::ShipEditor`. Purpose: assemble player ship designs and persist them as
schematics.

Layout: left panel (220 px wide) — Back button, tab strip (**Blocks** / **Ships**),
block palette (scrollable, one button per registered block type, with block icon layers +
name) or ship-list (saved schematics, click to load); center grid area; right panel
(250 px) — selected block properties (Name/Cost/Mass/HP), instructions, **Deploy** button
(stores the assembled ship as `DeployPayload.current_ship` — consumed by the next mission
spawn as the player's ship), **Start Game**.

Mechanics:

- Virtual grid, z = editor grid (`GRID_SIZE`), camera pans via left-mouse drag
  (`pan_speed = ortho.scale * 0.5`), zoom clamps `[0.1, 5.0]`
  (inconsistent with the game's `[0.2, 30.0]`).
- Placement requires face-connected placement onto the existing voxels
  (`check_placement_validity`), except an empty grid accepts anything.
- **Right-click deletes** blocks at cursor.
- Save triggers: any block add/remove writes `ShipSaveChanged` (`handle_ship_saving`),
  which requires ≥ 2 blocks and ≥ 1 Core; it writes `<filename>.json` in the schematics
  directory (see save.md) and emits `ShipSavesChanged` to refresh the ship list UI.
- Blocks listed in the editor are `Block` entities spawned without physics; they are
  tagged `SceneEntity(ShipEditor)`.
- Editor does not place `Ammo`/`Storage`/`Energy`/`Thruster` feature children — only
  `Texture` and `Turret` sprite layers are reproduced; everything else is dropped.

Ship list: `refresh_ship_list` reads `save_dir()/schematics/*.json` and rebuilds the UI on
`ShipSavesChanged` (and `OnEnter(ShipEditor)`).

Deletion nuance: `handle_block_deletion` deletes the block whose grid-anchor matches the
cursor; `remove_block_at` removes all voxels of that block.

## Pause

`SceneState::Game` or `ShipEditor`, toggled by `Escape`, mapped to
`PauseState::{Playing, Paused}` (`OnEnter(Paused)` pauses `Time<Virtual>`, `OnExit`
unpauses). Menu: **Resume**, **Settings**, **Back to Home**.
In `SceneState::Game` with `BattleMode::Campaign`, **Back to Home** triggers `SaveWorldEvent`
and defers the scene transition via `PendingExitAction::Home` until capture completes.
Application exit is available only from **Home**.

## Settings

Global settings UI (`SettingState::Open`), available from Home and Pause; modal absolute
panel (opacity 0.8). Controls:

- Volume — ±10 % linear (`GlobalVolume`), persists for the session only.
- Bloom — toggles `Bloom.intensity` between `0.3` (default) and `0.0` (OFF).
- Data (Home only) — **Reset Mission / Campaign** with 2-step confirmation (**Confirm Reset** /
  **Cancel**); deletes `battle.ron` and `campaign.ron` while keeping `designs/*.ron` intact,
  and refreshes Home's **Continue** label back to **Play** on close.
- Close.

Values are not written to disk; all-reset on restart.

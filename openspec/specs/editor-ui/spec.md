# Editor UI

## Purpose

The modular ship editor tracks designs by id, debounces RON saves, authors AI profiles, and exposes save slots and gameplay HUDs.

## Requirements
### Requirement: The editor is split into feature modules

`scene/editor.rs` SHALL be replaced by `scene/editor/{mod,ui,input,ship_list,save,ai_panel,grid}.rs` with the same behavior and no single 1,400-line file.

#### Scenario: Editor features resolve by module
- **WHEN** the editor is opened
- **THEN** placement, deletion, camera, ship list, save, AI, and grid all resolve through their modules

### Requirement: The selected design is tracked by id

The editor SHALL store `selected_ship_id: Option<Uuid>` and SHALL NOT always edit the first ship.

#### Scenario: Third design selected
- **WHEN** the user selects the third design
- **THEN** edits and saves apply to that design id

### Requirement: Design saving is debounced and explicit

The editor SHALL emit one `ShipDesignChanged` message, autosave after 500 ms of inactivity, provide an explicit Save action, and write RON only. `ShipSaveChanged` and `ShipSavesChanged` SHALL not exist.

#### Scenario: Dragging several blocks
- **WHEN** the user drags several blocks
- **THEN** one debounced save occurs, not one per edit

### Requirement: The AI panel edits profiles

The editor AI panel SHALL display `Doctrine` selector buttons (`Auto`, `Brawler`, `Skirmisher`, `Sniper`, `Support`, `Escort`), highlight the active doctrine, update `EditorState.doctrine`, and round-trip `ShipDesign.doctrine` through RON without rule lists or RON preview text.

#### Scenario: Doctrine round-trip
- **WHEN** a designer selects a `Doctrine`, saves the ship, and reopens it
- **THEN** the saved `Doctrine` is loaded and highlighted in the AI panel

### Requirement: Save slots are usable

The UI SHALL present five slots with timestamp, mission, ship count, and elapsed time; occupied slots SHALL require overwrite confirmation; write and read errors SHALL be shown to the player.

#### Scenario: Overwrite slot 2
- **WHEN** saving over slot 2
- **THEN** the player confirms and the slot metadata updates

### Requirement: Gameplay HUDs are minimal and event-driven

The game SHALL show power (production, demand, efficiency) and cargo (used, capacity) for the controlled ship, and editor stats SHALL show the same values, updating on change rather than every frame.

#### Scenario: Block destroyed
- **WHEN** a block is destroyed
- **THEN** power and cargo HUD values update from the change event

### Requirement: Deploy uses the current grid

Play and Deploy SHALL build a fresh validated payload from the current editor grid; a stale `DeployPayload` SHALL not be used.

#### Scenario: Move then deploy
- **WHEN** a block is moved and Deploy is pressed
- **THEN** the new layout is spawned

### Requirement: Settings are debounced and merged

Settings saves SHALL be debounced and merged with defaults; one settings source of truth (`GameSetting`) SHALL exist and include `ai_level: u8` (`1..=10`). The Settings UI overlay SHALL provide an `AI Skill` `[-]` / `[+]` stepper displaying the current level (`1..=10`) and updating `GameSetting.ai_level`.

#### Scenario: Rapid settings changes
- **WHEN** several settings including `ai_level` change quickly
- **THEN** one debounced write occurs and unspecified values keep defaults

#### Scenario: AI Skill stepper clamps to 1..=10
- **WHEN** the player clicks `-` at level `1` or `+` at level `10` in the Settings overlay
- **THEN** `GameSetting.ai_level` remains clamped within `1..=10` and the displayed value matches

### Requirement: Layout constants replace magic numbers

Panel widths and repeated layout values SHALL use named constants (for example 220 and 250) and respond to window width.

#### Scenario: Window resize
- **WHEN** the window is resized
- **THEN** panels keep their intended proportions without scattered literals

### Requirement: Blocks rotate before placement

The editor SHALL hold the pending placement rotation as `Rot90` in `EditorState`. The `rotate_ccw` and `rotate_cw` bindings SHALL rotate it counter-clockwise and clockwise. The rotation SHALL apply to the preview, the placement anchor (rotated footprint size / 2), validity checks, and the placed `BlockPlacement`. Bindings SHALL be read from the `keys` section of `setting.ron` (defaults `KeyQ` and `KeyE`) and merged with defaults.

#### Scenario: Rotate then place
- **WHEN** the user presses `E` once and places a `Rect((1,3))` block
- **THEN** the placement stores `Rot90::R270` and occupies three cells along the x axis

#### Scenario: Missing keys use defaults
- **WHEN** `setting.ron` has no `keys` section
- **THEN** rotation uses `KeyQ` and `KeyE`

### Requirement: Placement preview updates only on change

`update_placement_preview` SHALL run only when the hovered grid cell, selected block, pending rotation, or editor grid changes. It SHALL keep one persistent preview entity and update its transform, tint, and visual children in place, rebuilding children only when the selected block changes.

#### Scenario: Idle cursor
- **WHEN** the cursor stays on the same cell with the same block and rotation
- **THEN** the preview system does not run and no preview entity is spawned or despawned

#### Scenario: Cursor moves
- **WHEN** the hovered cell changes
- **THEN** the existing preview entity moves and its validity tint updates without being respawned

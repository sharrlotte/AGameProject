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

The AI panel SHALL list rules, allow add, remove, and reorder, edit priority, condition, behavior, and cooldown, show validation, apply presets, show the current behavior preview, and round-trip through RON.

#### Scenario: Profile round-trip
- **WHEN** a profile is saved and reloaded
- **THEN** rules and presets match

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

Settings saves SHALL be debounced and merged with defaults; one settings source of truth SHALL exist.

#### Scenario: Rapid settings changes
- **WHEN** several settings change quickly
- **THEN** one write occurs and unspecified values keep defaults

### Requirement: Layout constants replace magic numbers

Panel widths and repeated layout values SHALL use named constants (for example 220 and 250) and respond to window width.

#### Scenario: Window resize
- **WHEN** the window is resized
- **THEN** panels keep their intended proportions without scattered literals

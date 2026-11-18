## ADDED Requirements

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

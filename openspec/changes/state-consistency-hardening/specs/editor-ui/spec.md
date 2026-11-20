## MODIFIED Requirements

### Requirement: Deploy uses the current grid
Deploy SHALL build a fresh validated payload from the current editor grid; a stale `DeployPayload` SHALL NOT be used. Play SHALL continue the last save or create a new save and SHALL NOT deploy the current grid or overwrite a save with a fresh skirmish. Deploy SHALL upsert the resulting `ShipDesign` into `DeployPayload.fleet` by name: an entry with the same name SHALL be replaced, otherwise the design SHALL be appended. The Deploy button SHALL be disabled only while the current grid matches the fleet entry of that name, comparing `name`, `doctrine`, and each block's position, type, and rotation while ignoring `BlockPlacement.id`.

#### Scenario: Move then deploy
- **WHEN** a block is moved and Deploy is pressed
- **THEN** the new layout replaces the fleet entry with that name and is spawned

#### Scenario: New design appends
- **WHEN** a design whose name is not in the fleet is deployed
- **THEN** the design is appended to the fleet

#### Scenario: Edited design re-enables Deploy
- **WHEN** a fleet entry exists but the current grid differs in doctrine or block layout
- **THEN** the Deploy button is enabled and deploying replaces that entry

#### Scenario: Play from editor continues the save
- **WHEN** the editor was opened from Pause inside Slot A and the player clicks Play
- **THEN** Slot A is restored and no fresh battle from the current editor grid is spawned

## MODIFIED Requirements

### Requirement: Deploy uses the current grid

Play and Deploy SHALL build a fresh validated payload from the current editor grid; a stale `DeployPayload` SHALL NOT be used. Deploy SHALL upsert the resulting `ShipDesign` into `DeployPayload.fleet` by name: an entry with the same name SHALL be replaced, otherwise the design SHALL be appended. The Deploy button SHALL be disabled only while the current grid matches the fleet entry of that name, comparing `name`, `doctrine`, and each block's position, type, and rotation while ignoring `BlockPlacement.id`.

#### Scenario: Move then deploy
- **WHEN** a block is moved and Deploy is pressed
- **THEN** the new layout replaces the fleet entry with that name and is spawned

#### Scenario: New design appends
- **WHEN** a design whose name is not in the fleet is deployed
- **THEN** the design is appended to the fleet

#### Scenario: Edited design re-enables Deploy
- **WHEN** a fleet entry exists but the current grid differs in doctrine or block layout
- **THEN** the Deploy button is enabled and deploying replaces that entry

## ADDED Requirements

### Requirement: The editor exposes the deployed fleet

The editor SHALL display every deployed fleet ship with a removal action. Removal SHALL delete only the fleet entry, SHALL NOT delete the design from `ShipCatalog`, and SHALL persist the updated fleet.

#### Scenario: Remove a deployed ship
- **WHEN** the player removes a ship from the fleet list
- **THEN** that ship is absent from the fleet and from the next battle, while the design remains in the editor ship list

#### Scenario: Fleet list reflects upsert
- **WHEN** a design is deployed with a name already in the fleet
- **THEN** the fleet list shows one updated entry for that name, not two entries

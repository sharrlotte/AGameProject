# Deploy Fleet

## Purpose

The deployed fleet is the sole source of player ships. Missions define the battlefield and enemies only; the fleet is configured, persisted, and managed by the player.

## Requirements

### Requirement: The deployed fleet is the sole source of player ships

The system SHALL spawn every team-0 ship from `DeployPayload.fleet`. A mission definition SHALL NOT declare a player ship; its `spawn` field is only the fleet's anchor. Each fleet entry SHALL produce exactly one ship with `Team::new(DEFAULT_TEAM_ID)`, regardless of any team value in mission data.

#### Scenario: Fleet of two designs
- **WHEN** the deployed fleet contains two designs and Quick Battle is loaded
- **THEN** exactly two team-0 ships are spawned, one per fleet design

#### Scenario: Mission has no player field
- **WHEN** a mission definition contains no `player` field
- **THEN** the mission still loads and its player ships come from the deployed fleet

### Requirement: Fleet ships spawn in a configurable line formation

Fleet ships SHALL be placed on a line centered on the mission `spawn` anchor. Each design's hull extent SHALL be measured from its block layout, and hulls SHALL be placed edge-to-edge with `fleet_spacing` from `game.ron` as the gap between hulls, never the distance between centers. Asymmetric hulls SHALL use their own minimum and maximum cell offsets relative to the ship origin. The offset calculation SHALL be a pure function of the anchor, hull extents, and gap.

#### Scenario: Large and small hulls do not overlap
- **WHEN** a fleet contains a tall design and a short design at anchor `A` with gap `G`
- **THEN** the distance between the tall hull's edge and the short hull's edge is exactly `G`

#### Scenario: Three hulls centered on the anchor
- **WHEN** three identical hulls of height `H` spawn at anchor `A` with gap `G`
- **THEN** their center offsets are `-(H + G)`, `0`, and `H + G`

#### Scenario: Gap comes from config
- **WHEN** `fleet_spacing` changes in `game.ron`
- **THEN** the next battle uses the new gap without a code change

### Requirement: Default fleet is seeded from game config

At startup, when no persisted fleet exists, the system SHALL resolve `default_fleet` names from `game.ron` against `ShipCatalog` and use the resolved designs as the fleet. Unresolved names SHALL be skipped with a warning and SHALL NOT prevent startup.

#### Scenario: Fresh install fleet
- **WHEN** the game starts with no persisted fleet and `default_fleet` is `["Interceptor", "B2"]`
- **THEN** the fleet contains the `Interceptor` and `B2` designs from the catalog

#### Scenario: Unknown configured design
- **WHEN** `default_fleet` names a design that is not in the catalog
- **THEN** that name is skipped, a warning names it, and the remaining designs still form the fleet

### Requirement: The fleet is persisted and restored

The fleet SHALL be stored as a versioned RON document at `<data_dir>/fleet.ron` using the versioned, atomic document writer. Deploy and remove actions SHALL rewrite it. A persisted fleet SHALL replace the config defaults; config entries SHALL NOT be merged into it.

#### Scenario: Deploy survives restart
- **WHEN** a design is deployed and the game is restarted
- **THEN** the fleet loaded at startup is the persisted fleet, not the config defaults

#### Scenario: Persisted fleet wins over config
- **WHEN** a persisted fleet exists and `game.ron` lists different default names
- **THEN** the persisted fleet is used unchanged

### Requirement: An empty fleet is recovered

An empty persisted fleet SHALL NOT be accepted. At startup, an empty fleet SHALL be replaced by the config defaults and a warning SHALL be logged.

#### Scenario: Empty fleet document
- **WHEN** `<data_dir>/fleet.ron` contains an empty fleet
- **THEN** the fleet is restored from `default_fleet` and a warning is logged

### Requirement: Removing a ship does not delete its design

Removing a ship from the fleet SHALL change fleet membership only. The design SHALL remain in `ShipCatalog` and SHALL be deployable again.

#### Scenario: Remove then redeploy
- **WHEN** a player removes a deployed design and later deploys it again from the editor
- **THEN** the design is available and the fleet contains it again

# debris Specification

## Purpose
TBD - created by archiving change debris-bodies. Update Purpose after archive.
## Requirements
### Requirement: Detached components become independent debris bodies

After a grid mutation, every connected component that does not contain the core SHALL leave the ship and become an independent debris body. When the core is destroyed, every remaining component SHALL become debris, `ShipDestroyed` SHALL be emitted, and the ship entity SHALL be despawned. Each debris body SHALL own a `BlockGrid` containing only its component and a world transform, rotation, and velocity derived from the parent at the moment of detaching.

#### Scenario: Mid-ship block destroyed
- **WHEN** a block connecting a side section to the core is destroyed
- **THEN** the side section leaves the ship grid, becomes a debris body at its world position and rotation, and the ship collider excludes its hitboxes

#### Scenario: Core destroyed
- **WHEN** the core block is destroyed
- **THEN** the ship entity despawns, `ShipDestroyed` is emitted, and every surviving component becomes a debris body

#### Scenario: Blocks move with their body
- **WHEN** a component becomes debris under a rotated or moving parent
- **THEN** every block's world position and rotation is unchanged at the moment of detaching, and its `BlockOwner.body` names the new debris body

#### Scenario: No duplication
- **WHEN** a component becomes debris
- **THEN** each block exists in exactly one `BlockGrid` and exactly one entity has its `BlockRef`

### Requirement: Debris bodies are inert grid bodies

A debris entity SHALL carry `Debris`, `BlockGrid`, `Team`, physics components, and `Lifetime`, and SHALL NOT carry `Ship`, `ShipStats`, `AiRuntime`, or `Throttle`. Ship list, AI, HUD, power grid, turret, booster, cargo pickup, and mission outcome systems SHALL ignore debris. Blocks in a debris body SHALL keep `Health` and remain damageable.

#### Scenario: Debris is not a ship
- **WHEN** a debris body exists during a battle
- **THEN** it is absent from the ship list, is not targeted as a ship, does not fire turrets, does not produce or consume power, and cannot collect drops

#### Scenario: Debris blocks take damage
- **WHEN** a hostile bullet hits a debris block
- **THEN** the block loses health through the normal damage pipeline

### Requirement: Debris blocks remain destructible and split recursively

Destroying a block on a debris body SHALL remove it from that body's `BlockGrid`, recompute connected components, and create a new debris body for every resulting component that does not keep the current entity. A body whose grid becomes empty SHALL be despawned.

#### Scenario: Debris splits
- **WHEN** a block connecting two halves of a debris body is destroyed
- **THEN** the two halves become separate debris bodies and each has its own collider

#### Scenario: Debris shrinks without splitting
- **WHEN** a block on the edge of a debris body is destroyed
- **THEN** the body keeps its remaining blocks and rebuilds its collider and mass

#### Scenario: Last block destroyed
- **WHEN** the last block of a debris body is destroyed
- **THEN** the body despawns

### Requirement: Debris inherits the parent body's momentum

A new debris body SHALL receive the parent's linear velocity plus the angular contribution at the component centroid, and the parent's angular velocity. A debris body created by a split SHALL inherit the parent debris body's current velocity and angular velocity.

#### Scenario: Spinning ship throws debris
- **WHEN** a component detaches from a ship with non-zero angular velocity
- **THEN** the debris body moves with the parent linear velocity plus `omega x r` and keeps the parent angular velocity

#### Scenario: Split keeps momentum
- **WHEN** a debris body splits while moving
- **THEN** both resulting bodies keep the parent velocity and angular velocity

### Requirement: Debris expires into loot

A debris body SHALL receive a `Lifetime` drawn from a configured range whose default is 30 to 60 seconds. On expiry, every remaining block SHALL emit its `def.drops` at its world position for a non-player team, and the body SHALL despawn. Player-team debris SHALL emit no drops. Expiry SHALL NOT deal damage or apply an impulse. A split SHALL inherit the parent's remaining lifetime; a component detached from a ship SHALL roll a fresh lifetime.

#### Scenario: Wreck expires
- **WHEN** a non-player debris body's lifetime finishes
- **THEN** the blocks' configured drops spawn at their world positions and the body despawns

#### Scenario: Player wreck yields nothing
- **WHEN** a player-team debris body expires
- **THEN** no drops spawn and the body despawns

#### Scenario: Split does not extend life
- **WHEN** a debris body with 5 seconds remaining splits
- **THEN** every resulting body has at most 5 seconds remaining

#### Scenario: Expiry is not a hazard
- **WHEN** a debris body expires next to a ship
- **THEN** the ship receives no damage and no impulse

### Requirement: Debris is ship-collidable and radar-invisible

Debris SHALL use `GROUP_DEBRIS`. Ships SHALL collide with debris, debris SHALL collide with debris, radar SHALL NOT treat debris as a ship target, and bodies SHALL NOT merge on contact. Debris blocks SHALL keep their team block hitbox group so hostile fire can damage them.

#### Scenario: Ship hits a wreck
- **WHEN** a ship collides with a debris body
- **THEN** both bodies respond physically

#### Scenario: Radar ignores debris
- **WHEN** a radar scan runs with a debris body in range
- **THEN** the debris body is not returned as a ship contact

#### Scenario: Contact does not merge
- **WHEN** a debris body rests against a ship or another debris body
- **THEN** neither grid, collider, nor ownership changes

### Requirement: Debris behavior has automated tests

Tests SHALL cover detachment from a ship, core-loss wrecking, world transform preservation, recursive split, empty-body despawn, momentum inheritance, expiry drops with the team gate, inherited remaining lifetime, collision group membership, radar exclusion, save round-trip with remaining lifetime, and absence of ship stats changes.

#### Scenario: Debris suite runs
- **WHEN** `cargo test` runs
- **THEN** the debris tests pass and prove no block belongs to two bodies and no debris body carries ship-only components


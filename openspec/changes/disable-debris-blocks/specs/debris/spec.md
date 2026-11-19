## MODIFIED Requirements

### Requirement: Debris bodies are inert grid bodies

A debris entity SHALL carry `Debris`, `BlockGrid`, `Team`, physics components, and `Lifetime`, and SHALL NOT carry `Ship`, `ShipStats`, `AiRuntime`, or `Throttle`. Every block owned by a `Debris` entity SHALL carry `Disabled` so ship list, AI, HUD, power grid, turret, shooting, radar, booster, thruster, cargo pickup, and mission outcome systems ignore debris and its blocks. Blocks in a debris body SHALL keep `Health` and remain damageable.

#### Scenario: Debris is not a ship
- **WHEN** a debris body exists during a battle
- **THEN** it is absent from the ship list, is not targeted as a ship, its blocks are marked `Disabled`, it does not fire turrets, does not run radars or boosters, does not produce or consume power, and cannot collect drops

#### Scenario: Debris blocks take damage
- **WHEN** a hostile bullet hits a debris block
- **THEN** the block loses health through the normal damage pipeline

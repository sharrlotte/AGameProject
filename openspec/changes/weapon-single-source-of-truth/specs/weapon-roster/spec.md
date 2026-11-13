## MODIFIED Requirements

### Requirement: Weapon roster defines multiple playable weapons
The simulation SHALL define the playable weapon roster through a single unified `WeaponDefinition` owned
by the config foundation, containing at least five weapons — `m16`, `m1911`, `mp5`, `mp7`, and `scar` — each
with distinct stats.

#### Scenario: Roster contains five weapons

- **WHEN** the unified weapon definition is loaded
- **THEN** the definition contains `m16`, `m1911`, `mp5`, `mp7`, and `scar`

#### Scenario: Stats differ by class

- **WHEN** comparing the converted guns
- **THEN** each has distinct fire rate, damage, and magazine size

### Requirement: Presentation profile exists for each weapon
The unified `WeaponDefinition` SHALL provide presentation data — model path, sockets, poses, recoil, reload,
swap, and grip — for each weapon alongside its stats.

#### Scenario: Profiles match roster

- **WHEN** the unified weapon definition is loaded
- **THEN** each weapon has presentation data with its stats

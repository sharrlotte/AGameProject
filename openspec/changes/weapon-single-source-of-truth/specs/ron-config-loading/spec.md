## MODIFIED Requirements

### Requirement: Feature-crate config ownership
Each feature crate SHALL own and register its domain configuration structs rather than centralizing them in
`data`. Weapon configuration SHALL be owned by the shared config foundation through a single unified
`WeaponDefinition`; feature crates SHALL NOT register separate weapon roster catalogs.

#### Scenario: Weapon config ownership moves to the config foundation

- **WHEN** the weapon configuration is registered
- **THEN** it is owned by the shared config foundation rather than the combat feature crate

## MODIFIED Requirements

### Requirement: Quick Battle showcases both new roles

The Quick Battle mission SHALL define a spawn anchor and an enemy list only; it SHALL NOT declare a player ship. The player fleet SHALL be the config-driven default fleet (`Interceptor` and `B2` on a fresh install), spawned at the mission anchor. The enemy roster SHALL be resolved from the mission's `enemies` list.

#### Scenario: Quick Battle roster
- **WHEN** Quick Battle is loaded with a fresh default fleet
- **THEN** the player ships resolve to `Interceptor` and `B2` and the enemy ships resolve to the mission's configured designs

#### Scenario: Custom fleet replaces defaults
- **WHEN** Quick Battle is loaded with a persisted custom fleet
- **THEN** the player ships are the custom fleet designs and no default designs are added

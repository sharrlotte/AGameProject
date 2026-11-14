## ADDED Requirements

### Requirement: Projectile spawn origin computes relative to shooter translation

When a weapon fires, projectile muzzle position SHALL be calculated relative to the shooter's actual dynamic translation ($Y = \text{translation}.y + \text{aim\_height}$), adding the forward aim vector multiplied by the weapon definition's `muzzle_offset`.

#### Scenario: Bullet spawns at shooter dynamic elevation
- **WHEN** a weapon is fired from a shooter entity at position $(X, Y, Z)$
- **THEN** the initial projectile translation has $Y = \text{translation}.y + \text{aim\_height}$ and is offset forward along the aim direction by `muzzle_offset`

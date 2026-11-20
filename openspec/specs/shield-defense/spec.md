# shield-defense Specification

## Purpose
TBD - created by archiving change shield-support. Update Purpose after archive.
## Requirements
### Requirement: Shield Configuration and Block Feature
The engine SHALL support configuring shield features on blocks via `FeatureConfig::Shield` in block configuration files (`block.ron`). A shield configuration SHALL specify shape geometry (`Circle` with `radius` or `Plane` with `width`, `thickness`, and `offset`), `max_hp`, `cooldown` (seconds), `regen_rate` (hp per second), and optional power drain `demand`.

#### Scenario: Block with Circle Shield spawns emitter
- **WHEN** a ship is instantiated with a block featuring a `Circle` shield
- **THEN** a `ShieldEmitter` component is inserted on the block entity with configured radius, max health, cooldown, and regen rate

#### Scenario: Block with Plane Shield spawns emitter
- **WHEN** a ship is instantiated with a block featuring a `Plane` shield
- **THEN** a `ShieldEmitter` component is inserted on the block entity oriented to the block's placement transform and offset in front of the emitter

### Requirement: Shield Collision Filtering and Ship Permeability
Shield colliders SHALL be non-solid sensors assigned to the owning team's block hitbox collision group (`1 << (16 + team_id)`). Ships SHALL pass freely through shields without physics response, friendly projectiles SHALL pass through without collision, and hostile projectiles SHALL trigger collision events with the shield.

#### Scenario: Ship flies through an enemy or friendly shield
- **WHEN** a ship collider moves through an active shield sensor
- **THEN** no collision impulses or movement hindrances are applied to the ship

#### Scenario: Friendly projectile fires through own shield
- **WHEN** an allied ship fires a bullet from behind or within its own shield
- **THEN** the bullet passes through the shield collider without despawning or damaging the shield

#### Scenario: Hostile bullet strikes active shield
- **WHEN** an enemy projectile collides with an active shield sensor
- **THEN** a collision event is registered against the shield, the projectile is consumed (unless piercing), and damage is applied to the shield

### Requirement: Shield Lifecycle State Machine
Shields SHALL follow a state machine consisting of `Active`, `Depleted`, and `Regenerating` states. When `Active` shield health reaches zero or below, the shield SHALL transition to `Depleted`, disabling its collider and visual effect for the configured cooldown duration. Once the cooldown expires, the shield SHALL transition to `Regenerating` at `regen_rate * supply` HP per second, re-enabling the shield and transitioning to `Active` only when health reaches `max_hp`.

#### Scenario: Shield absorbs damage and stays active
- **WHEN** an active shield with 100 HP receives 30 damage
- **THEN** the shield health becomes 70 HP and the shield remains active

#### Scenario: Shield breaks and enters cooldown
- **WHEN** an active shield with 20 HP receives 50 damage
- **THEN** the shield health drops to 0, excess damage is discarded (0 bleed-through), the shield transitions to `Depleted`, and its collider and visual are disabled

#### Scenario: Depleted shield recovers to regenerating state
- **WHEN** a depleted shield completes its configured cooldown duration
- **THEN** the shield transitions to `Regenerating` with 0 HP and begins charging health

#### Scenario: Regenerating shield reactivates at full health
- **WHEN** a regenerating shield charges until its health equals or exceeds `max_hp`
- **THEN** the shield transitions to `Active`, re-enabling its collider and visual effect

### Requirement: Beam Clipping and Missile Containment
Continuous laser beams and missile impacts SHALL be blocked at the shield boundary. Laser beams SHALL clip their ray and visual extent at the first hit collider, dealing damage to the shield without continuing through to hull blocks. Missile hits on shields SHALL deal direct damage to the shield entity and SHALL NOT spawn an unshielded spherical blast sensor.

#### Scenario: Continuous laser beam strikes shield
- **WHEN** a continuous laser beam intersects an active shield
- **THEN** the laser beam ray terminates at the shield surface, deals tick damage to the shield, and renders a truncated visual beam

#### Scenario: Missile strikes shield
- **WHEN** a guided missile collides with an active shield sensor
- **THEN** the missile detonates on the shield, its blast damage is applied directly to the shield, and no external blast sensor collider is spawned to damage hull blocks behind it

### Requirement: Power Grid Integration
Shield emitters with configured power demand SHALL register as power consumers on the ship power grid. If available power supply falls to starvation threshold or lower (`Supply <= STARVATION_THRESHOLD`) or the emitter block is `Disabled`, the shield collider and visual SHALL deactivate. During regeneration, recharge speed SHALL scale proportionally with `Supply`.

#### Scenario: Power loss drops active shield
- **WHEN** a ship's power grid supply drops below starvation threshold
- **THEN** the shield immediately deactivates its collider and visual

#### Scenario: Low power slows shield regeneration
- **WHEN** a shield regenerates with a power supply ratio of 0.5
- **THEN** health regenerates at half of the base `regen_rate`

### Requirement: Procedural WGSL Shader Presentation
Active and regenerating shields SHALL be rendered using a custom 2D WGSL material shader positioned at `LAYER_SHIELD` ($Z = 160.0$), rendering above hull blocks ($Z = 10.0$), turret barrels ($Z = 50.0$), and projectiles ($Z = 100.0$). The shader SHALL receive the owning ship's team color, elapsed time, shape type, and a health ratio uniform (`hp / max_hp`), rendering an animated energy boundary that fades or dims as health depletes and becomes invisible when depleted or unpowered. Circle shields SHALL reconstruct a hemispherical surface from the flat mesh UV so that the rim and honeycomb respond to reconstructed surface curvature; plane shield presentation SHALL be unchanged.

#### Scenario: Active shield visual reflects health, team color, and envelope depth
- **WHEN** an active shield is rendered on a ship with mounted turrets
- **THEN** the custom WGSL shader renders the shield mesh at $Z = 160.0$ tinted with the ship's team color, wrapping above both the hull structure and turret barrels with opacity/glow reflecting current health percentage

#### Scenario: Circle shield reconstructs a hemisphere from the disc UV
- **GIVEN** an active circle shield rendered from its flat `Mesh2d`
- **WHEN** the WGSL fragment shader evaluates a pixel at normalized disc coordinate `p` with radius `r`
- **THEN** the shader derives a surface normal `(p.x, p.y, sqrt(1 - r²))` and uses its depth component as the source of the rim, honeycomb curvature, and interior body

#### Scenario: Rim remains aligned with the physics boundary
- **GIVEN** a circle shield configured with radius `R`
- **WHEN** the rim is rendered
- **THEN** its screen-space radius is `0.985 × R`, matching the `Collider::ball(R)` boundary, and the silhouette is antialiased rather than hard-discarded

#### Scenario: Upper-left key is the only directional light cue
- **GIVEN** the circle shield's thin rim
- **WHEN** rim brightness is evaluated around the circumference
- **THEN** it is brightest in the upper-left and dimmest in the lower-right according to one consistent key direction, and no broad specular highlight or gradient wash is rendered

#### Scenario: Honeycomb foreshortens toward the silhouette
- **GIVEN** an active circle shield's honeycomb wireframe
- **WHEN** cell coordinates are computed
- **THEN** they are sampled in surface arc length `asin(r)` so cells compress radially toward `r = 1`, and the traveling ripple phase is evaluated in the same arc-length coordinate

#### Scenario: Circle shield interior has a restrained body tint
- **GIVEN** an undamaged active circle shield
- **WHEN** a pixel inside a hex cell is rendered with `AlphaMode2d::Blend`
- **THEN** it receives an interior tint of at most 5% that is strongest at the dome center and fades to zero at the rim, tinting the hull behind rather than adding light

#### Scenario: Plane shield presentation is unchanged
- **GIVEN** an active plane shield
- **WHEN** the WGSL fragment shader evaluates it
- **THEN** it uses the existing flat honeycomb wireframe, front boundary rim, side falloff, and transverse ripple with no hemispherical reconstruction and no interior body tint

#### Scenario: Depleted or unpowered shield renders nothing
- **GIVEN** a circle shield that is depleted, regenerating but hidden, or unpowered
- **WHEN** visuals are updated
- **THEN** the barrier is hidden and no dome, rim, honeycomb, or body tint is rendered


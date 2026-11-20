## MODIFIED Requirements

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

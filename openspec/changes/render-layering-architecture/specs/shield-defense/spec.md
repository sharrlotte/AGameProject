## MODIFIED Requirements

### Requirement: Procedural WGSL Shader Presentation
Active and regenerating shields SHALL be rendered using a custom 2D WGSL material shader positioned at `LAYER_SHIELD` ($Z = 160.0$), rendering above hull blocks ($Z = 10.0$), turret barrels ($Z = 50.0$), and projectiles ($Z = 100.0$). The shader SHALL receive the owning ship's team color, elapsed time, shape type, and a health ratio uniform (`hp / max_hp`), rendering an animated energy boundary that fades or dims as health depletes and becomes invisible when depleted or unpowered.

#### Scenario: Active shield visual reflects health, team color, and envelope depth
- **WHEN** an active shield is rendered on a ship with mounted turrets
- **THEN** the custom WGSL shader renders the shield mesh at $Z = 160.0$ tinted with the ship's team color, wrapping above both the hull structure and turret barrels with opacity/glow reflecting current health percentage

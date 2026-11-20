# render-layering Specification

## Requirements

### Requirement: Standardized Visual Layer Depth Constants
The game engine SHALL define standardized, non-overlapping float constants for visual depth in `core/src/constant.rs`. Visual entities and components SHALL be assigned depth according to this strata:
- `LAYER_DEBRIS: f32 = 2.0`
- `LAYER_HULL: f32 = 10.0`
- `LAYER_TURRET_BASE: f32 = 20.0`
- `LAYER_TURRET_BARREL: f32 = 50.0`
- `LAYER_OVERDRIVE_FIELD: f32 = 70.0`
- `LAYER_BULLET: f32 = 100.0`
- `LAYER_BEAM: f32 = 110.0`
- `LAYER_DROP: f32 = 120.0`
- `LAYER_SHIELD: f32 = 160.0`
- `LAYER_EXPLOSION: f32 = 200.0`
- `LAYER_FLASH_OVERLAY: f32 = 220.0`
- `LAYER_BLOCK_ICON: f32 = 300.0`

#### Scenario: Visual entities use standardized depth strata
- **WHEN** a ship with hull blocks, mounted turrets, an active shield, and booster aura is spawned in world space
- **THEN** hull blocks render at $Z = 10.0$, turret barrels render at $Z = 50.0$, booster aura renders at $Z = 70.0$, and the shield barrier renders at $Z = 160.0$

### Requirement: Turret Barrel Visual Layering
Turret barrel sprites SHALL be positioned at `LAYER_TURRET_BARREL` ($Z = 50.0$) in block configurations and entity spawns, rendering above the base hull ($Z = 10.0$) and below the booster aura ($Z = 70.0$) and shield envelopes ($Z = 160.0$).

#### Scenario: Turret barrel renders above hull but under shield
- **WHEN** a turret is mounted on a hull block covered by an active shield
- **THEN** the turret barrel texture is drawn at $Z = 50.0$, visually above the hull ($Z = 10.0$) and underneath the shield barrier ($Z = 160.0$)

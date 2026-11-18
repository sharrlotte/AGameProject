## ADDED Requirements

### Requirement: 1x4 Foreshadow railgun block configuration
The `foreshadow` block in `core/assets/config/base/block.ron` SHALL be configured as a `1×4` fixed spinal railgun.

#### Scenario: Foreshadow dimensions, rotation, and muzzle offset
- **WHEN** `core/assets/config/base/block.ron` is loaded
- **THEN** `foreshadow` SHALL have `shape: Rect((1, 4))`, `turn_speed: 0.0`, `barrels: [(0.0, 128.0)]`, and `aim_tolerance: Some(0.035)`

### Requirement: 1x4 Foreshadow pixel-art sprite
`core/assets/texture/block/foreshadow.png` SHALL be a `64×256` RGBA sprite matching the `1×4` aspect ratio and `AGENTS.md` art style.

#### Scenario: Sprite dimensions and transparent background
- **WHEN** `core/assets/texture/block/foreshadow.png` is inspected
- **THEN** it SHALL be `64×256` RGBA, upscaled 2× with `NEAREST` from `32×128`, with a transparent background (no square hull base) and dark space-steel railgun styling

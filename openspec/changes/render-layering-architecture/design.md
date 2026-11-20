## Context

In 2D top-down space combat, entities, weapon hardpoints, projectiles, shields, and overlays are rendered in a shared camera orthographic projection. Depth sorting relies on the Z component of `Transform.translation`. Currently, turret barrels are set to `z: 150.0` in `block.ron`, while shield barriers are spawned at `z = 100.0` (`LAYER_BULLET`), resulting in shields rendering beneath turret barrels.

## Goals / Non-Goals

**Goals:**
- Provide a standardized, non-overlapping float constants table in `core/src/constant.rs`.
- Ensure shield envelopes ($Z = 160.0$) render visually above hull ($Z = 10.0$), turret barrels ($Z = 50.0$), and bullets ($Z = 100.0$).
- Recalibrate booster overdrive fields to $Z = 70.0$, drawing over hull and turret blocks but beneath shields.
- Keep all block definitions in `block.ron` consistent with the layering strata.

**Non-Goals:**
- Introducing custom multi-camera Bevy `RenderLayers` or separate pass pipelines at this time.
- Modifying UI canvas / Egui rendering depth.

## Decisions

### Decision 1: Standardized Float Constants in `constant.rs`
- **Choice**: Direct `pub const LAYER_<NAME>: f32` constants in `core/src/constant.rs`.
- **Rationale**: Minimal migration friction, zero deserialization overhead, consistent with the existing codebase style.
- **Values**:
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

### Decision 2: Recalibrating Turret Barrels in `block.ron` to 50.0
- **Choice**: Change all turret barrel texture entries in `block.ron` from `z: 150.0` to `z: 50.0`.
- **Rationale**: Turrets must sit on top of the hull/base (`10.0`), but under the booster field (`70.0`), projectiles (`100.0`), and shield bubbles (`160.0`).

### Decision 3: Shields Above Projectiles and Turrets
- **Choice**: `LAYER_SHIELD` placed at `160.0` above `LAYER_BULLET` (`100.0`) and `LAYER_TURRET_BARREL` (`50.0`).
- **Rationale**: Shields act as outer protective domes enclosing the entire vessel. Hostile incoming bullets impact the surface of the shield envelope; friendly outgoing shots fire outward from within the dome.

## Risks / Trade-offs

- **[Risk] Existing RON files or test designs using hardcoded `150.0`** → **Mitigation**: Update all `z: 150.0` entries in `core/assets/config/base/block.ron` and search all tests/presets for stale Z constants.
- **[Risk] Transparent Overdrive Ring over Turrets** → **Mitigation**: With the dashed circle and no fill, the ring is only a thin perimeter line that does not obscure turret functionality.

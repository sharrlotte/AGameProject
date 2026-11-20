## 1. Standardize Layer Constants in `constant.rs`

- [ ] 1.1 Add and update visual depth constants in `core/src/constant.rs` (`LAYER_DEBRIS = 2.0`, `LAYER_HULL = 10.0`, `LAYER_TURRET_BASE = 20.0`, `LAYER_TURRET_BARREL = 50.0`, `LAYER_OVERDRIVE_FIELD = 70.0`, `LAYER_BULLET = 100.0`, `LAYER_BEAM = 110.0`, `LAYER_DROP = 120.0`, `LAYER_SHIELD = 160.0`, `LAYER_EXPLOSION = 200.0`, `LAYER_FLASH_OVERLAY = 220.0`, `LAYER_BLOCK_ICON = 300.0`)

## 2. Update Turret Barrels in Block Configuration

- [ ] 2.1 Update all turret barrel texture entries in `core/assets/config/base/block.ron` from `z: 150.0` to `z: 50.0`

## 3. Update Shield and Overdrive Field Systems

- [ ] 3.1 Update `spawn_shield_barriers` in `core/src/content/block/feature/shield/system.rs` to set barrier transform to `LAYER_SHIELD` (`160.0`)
- [ ] 3.2 Ensure `core/src/content/block/feature/power/field/` uses `LAYER_OVERDRIVE_FIELD` (`70.0`)

## 4. Documentation and Verification

- [ ] 4.1 Update `docs/combat.md` and `docs/simulation.md` to reflect the updated layer specifications
- [ ] 4.2 Run `cargo test --bin core` to verify all unit tests pass
- [ ] 4.3 Launch `cargo dev` in the background and verify visual rendering

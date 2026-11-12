## 1. Catalog data model

- [x] 1.1 Add `right_hand_offset_deg` and `left_hand_offset_deg` fields to `IkConfig` in `crates/presentation/src/anim/catalog/mod.rs`
- [x] 1.2 Add serde default functions returning `[0.0, 0.0, 0.0]` for the new fields
- [x] 1.3 Update `IkConfig::default()` to include the new fields

## 2. IK system per-hand offset

- [x] 2.1 Update `solve_chain` signature in `crates/presentation/src/anim/ik/mod.rs` to accept a per-hand `hand_offset: Quat`
- [x] 2.2 Update `solve_player_arms` to pass `right_hand_offset_deg` for right arm and `left_hand_offset_deg` for left arm
- [ ] 2.3 Construct `Quat` from each hand's offset degrees using `EulerRot::XYZ`

## 3. Config RON update

- [x] 3.1 Set starting `right_hand_offset_deg` and `left_hand_offset_deg` values in `game/assets/config/player/animation.ron`

## 4. Tests

- [x] 4.1 Add unit test asserting per-hand offsets are applied to correct arms
- [x] 4.2 Run targeted presentation catalog + IK tests

## 5. Empirical tuning

- [ ] 5.1 Run game and check hand/palm orientation
- [ ] 5.2 Adjust offset values based on visual feedback (repeat as needed)
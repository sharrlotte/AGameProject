## 1. Grip data model

- [x] 1.1 Add `GripHand` with optional `position`, `finger`, `palm`, `elbow`, `fingers`, `wrist_deg`, and `align_deg`
- [x] 1.2 Add `WeaponGripConfig` with `primary`, optional `support`, and `min_support_separation`
- [x] 1.3 Add finite/zero-length/curl validation and bare-value RON deserialization for optional vectors
- [x] 1.4 Move `FingerCurl` into the grip module with one scalar per bone

## 2. Grip resolution

- [x] 2.1 Convert mounted socket frames into weapon space
- [x] 2.2 Derive the weapon forward/up/right basis from the grip socket and muzzle hint
- [x] 2.3 Derive primary/support hands and merge authored fields one by one
- [x] 2.4 Fall back safely for missing/too-close foregrips and `support: None`
- [x] 2.5 Build `HandFrame`/`GripTarget`/`WeaponTargets` from resolved hands

## 3. IK

- [x] 3.1 Add per-hand wrist trims and weapon-space `align_deg` to `ArmTarget`
- [x] 3.2 Use grip-authored elbow poles instead of catalog pole vectors
- [x] 3.3 Apply per-hand finger curls after the arm solve
- [x] 3.4 Keep reload blending of the support hand to the magazine socket

## 4. Model spaces and catalog

- [x] 4.1 Add `ModelSpace::{Weapon, ForwardNegYUpZ}` and `model_transform()`
- [x] 4.2 Apply the model transform when posing the mounted scene
- [x] 4.3 Add socket overrides and resolve them once per mount
- [x] 4.4 Update `weapons.ron` with model spaces, socket overrides, and per-weapon grips
- [x] 4.5 Update `animation.ron` to the scalar curl fields

## 5. Ordering and debug

- [x] 5.1 Run idle relaxation in `PlayerAnimSet::Weapon` before IK
- [x] 5.2 Add cfg-gated `IkDebug` and `draw_ik_gizmo` behind `dev-ik-gizmo`

## 6. Verification and docs

- [x] 6.1 Add resolver, catalog, and model-space tests
- [x] 6.2 Run `cargo test -p presentation --features dev`
- [x] 6.3 Check with `dev` and `dev,dev-ik-gizmo` features
- [x] 6.4 Update `docs/09-player-asset-baseline.md`
- [x] 6.5 Calibrate M16 and support-hand grip in-game with the gizmo

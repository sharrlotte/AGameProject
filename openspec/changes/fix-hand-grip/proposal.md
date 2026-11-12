## Why

Hand/palm orientation is incorrect when holding weapons. Both hands share a single `hand_offset_deg` rotation, but the right and left hands need different (mirrored) orientations to wrap naturally around their respective grips (pistol grip vs. fore-end). The palms face the wrong direction, wrists twist awkwardly, and fingers don't curl around the weapon.

## What Changes

- Replace single `hand_offset_deg` in `IkConfig` with per-hand `right_hand_offset_deg` and `left_hand_offset_deg` fields.
- Update `solve_chain` to use the per-hand offset when solving each arm.
- Set starting offset values based on skeleton bind-pose analysis (palm normal, finger direction).
- Add empirical-tuning tasks to adjust offsets based on visual feedback.
- Add unit test asserting per-hand offsets are applied correctly.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `player-animation`: dual-hand weapon IK gains per-hand grip orientation offsets.

## Impact

- Affected code: `crates/presentation/src/anim/catalog/mod.rs` (IkConfig), `crates/presentation/src/anim/ik/mod.rs` (solve_chain), `game/assets/config/player/animation.ron` (offset values).
- No sim changes; purely presentation.
- Backward-compatible: new fields default to identity if not specified.
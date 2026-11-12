## Context

Dual-hand weapon IK solves both arms to weapon sockets (`Grip`/`Foregrip`). Hand rotation is set from `target.rotation * hand_offset_deg`, where `target.rotation` is the socket orientation (identity for both sockets) and `hand_offset_deg` is a single shared `[f32; 3]` array. Both hands receive the same offset, but the right and left hands need mirrored orientations to wrap naturally around the pistol grip vs. fore-end.

From headless Blender inspection of `player.glb`:
- Palm.R bind-pose rest rotation: (-0.092, 0.012, -0.001, 0.996) ≈ pitch -10.5°.
- Palm bone axes (armature space): X≈+Z (fingers), Y≈-X (palm normal toward body), Z≈-Y (down).
- Both sockets have identity rotation, so `hand_offset_deg` fully determines the hand orientation.

## Goals / Non-Goals

**Goals:**
- Let right and left hands use different rotation offsets.
- Restore bind-pose hand orientation as a starting point.
- Allow empirical tuning per hand via RON.

**Non-Goals:**
- Finger curl (zero-length finger tip bones cannot deform mesh procedurally; requires asset work).
- Per-weapon grip overrides (follow-up).
- Changing IK solver math or pole target behavior.

## Decisions

### D1. Split `hand_offset_deg` into per-hand fields
**Choice:** Add `right_hand_offset_deg` and `left_hand_offset_deg` (each `[f32; 3]`, default identity) to `IkConfig`. Keep `hand_offset_deg` for backward compatibility but ignore it if either new field is present.
**Why:** Right and left hands are mirror images; they need different grip orientations. Per-hand fields are the minimal change to `IkConfig`.
**Alternative considered:** Keep single field and compute mirrored offset internally. Rejected: doesn't allow independent tuning and hides the asymmetry.

### D2. Use per-hand offset in `solve_chain`
**Choice:** `solve_chain` receives the hand offset for its specific hand and passes it to `solve_arm` as `hand_offset`.
**Why:** `solve_chain` already knows which chain (right/left) it's solving. Passing the corresponding offset is straightforward.
**Alternative:** Compute offset in caller and pass as parameter. Rejected: same complexity, less encapsulation.

### D3. Default values
**Choice:** Both default to `[0.0, 0.0, 0.0]` (identity). Users must set non-zero values via RON.
**Why:** Safe default; existing behavior preserved when fields are unset.
**Alternative:** Default to bind-pose rotation (-10.5° pitch). Rejected: changes existing behavior without user opt-in.

## Risks / Trade-offs

- **Empirical tuning needed**: Correct offset values depend on mesh geometry; starting from identity may not look correct immediately. Mitigation: provide tuning tasks and instruct user to iterate.
- **Backward compat**: `hand_offset_deg` still parsed but overridden if new fields present. Mitigation: document clearly.
- **No finger curl**: Zero-length finger bones mean procedural curl won't deform mesh. Mitigation: document as asset-work follow-up.

## Migration Plan

1. Add `right_hand_offset_deg` and `left_hand_offset_deg` to `IkConfig`.
2. Update `solve_chain` signature and call sites to pass per-hand offset.
3. Update `animation.ron` with starting offset values (identity defaults).
4. Run targeted presentation tests.
5. Tune values based on visual feedback.

## Open Questions

- Starting offset values (identity default may need adjustment).
- Whether to deprecate `hand_offset_deg` entirely.
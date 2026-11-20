## Why

Circle shields currently read as a flat decal. `core/assets/shader/shield.wgsl` derives every effect from 2D UV coordinates: a fixed-width annulus at `dist = 0.985`, a uniform honeycomb `hex_coords(p * 14.0)`, and a radial ripple in flat `dist`. The `world_normal` and `world_position` varyings are declared but never used, and a flat `Mesh2d` has a constant normal anyway, so no curvature information ever reaches the fragment shader. The result is a sticker rather than the hemispherical energy bubble in the reference image.

The fix must not become a rendering-architecture change. The archived `render-layering-architecture` design explicitly lists multi-camera `RenderLayers` and separate passes as a **non-goal**, and the `shield-defense` spec pins the barrier at `LAYER_SHIELD` (`Z = 160.0`) above hull, turrets, and projectiles. Reconstructing the hemisphere from the disc UV inside the existing fragment shader meets the visual goal without touching cameras, meshes, colliders, or the layer table.

The updated `AGENTS.md` art style further constrains the result: grounded rather than cartoon-like, restrained cyan accents, one consistent upper-left light direction, no unnecessary gradients, small purposeful highlights, and every visible detail at least 2px. A broad fresnel wash and a large specular lobe would violate those rules, so the visual model uses a thin curvature-derived rim, an upper-left-weighted key on that rim only, flat wireframe cells, and no specular highlight.

## What Changes

- Rewrite only the circle branch (`shape_type < 0.5`) of `core/assets/shader/shield.wgsl` to reconstruct the visible hemisphere from the disc UV (`N = (p.x, p.y, sqrt(1 - r²))`) and derive the rim from its depth component instead of a fixed UV annulus.
- Keep the rim geometrically identical to the current `0.985` by placing it at `nz ≈ 0.173`, preserving alignment with the `Collider::ball(radius)` boundary.
- Sample the honeycomb in surface arc length (`asin(r)/r * p`) so cells foreshorten toward the silhouette. Keep the existing thin wireframe (no bevel or relief shading) and re-express the ripple over arc length so it travels at constant surface speed.
- Add a restrained 5% interior body tint derived from hemisphere depth, strongest at the dome center and fading at the rim, replacing strict 0% cell fill for circle shields only.
- Weight the thin rim with an upper-left key direction; add no specular lobe and no broad gradient.
- Replace the hard silhouette `discard` with `fwidth`-based antialiasing, matching the sibling `overdrive_field.wgsl`.
- Keep `AlphaMode2d::Blend`, the `ShieldMaterial` uniform layout, every Rust system, the plane branch, colliders, and `LAYER_SHIELD` unchanged.
- Tune the look with WGSL `const` values only — no new RON keys and no uniform fields. This is an explicit, temporary deviation from the data-driven preference in `AGENTS.md`; the rationale and promotion path are recorded in `design.md`.
- Amend the Visuals subsection of `docs/combat.md` to describe the new presentation model and the circle-only 0% → 5% fill change.

## Capabilities

### Modified Capabilities

- `shield-defense`: circle-shield visual presentation becomes a curvature-derived hemispherical bubble — thin rim aligned with the collider boundary, upper-left-lit, foreshortened honeycomb, arc-length ripple, and a restrained 5% interior body. Plane-shield visuals and all collision, lifecycle, and power behavior are unchanged.

## Impact

- **Dependencies**: none new. No new asset, camera, render layer, or plugin.
- **Files**: `core/assets/shader/shield.wgsl` (circle branch only); `docs/combat.md` (Visuals subsection).
- **Unchanged**: `component.rs` (`ShieldMaterial` / `alpha_mode`), `system.rs`, `mod.rs`, `block.ron`, `constant.rs`, colliders, `LAYER_SHIELD`, and the entire plane-shield branch.
- **Documentation**: `docs/combat.md`.
- **Config / save migration**: none. Existing saves and starter designs are unaffected.

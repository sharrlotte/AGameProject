## 1. Hemispherical circle shader

- [ ] 1.1 Add a WGSL `const` block to `core/assets/shader/shield.wgsl` for `HEX_DENSITY`, `RIM_NZ` (0.173), `RIM_WIDTH`, `KEY_LIGHT` (pre-normalized upper-left), `KEY_FLOOR`, `KEY_RANGE`, and `BODY_STRENGTH` (0.05), leaving `ShieldMaterial` and all RON files untouched
- [ ] 1.2 In the circle branch only (`shape_type < 0.5`), reconstruct the hemisphere normal `n = vec3(p, sqrt(max(0.0, 1.0 - r*r)))` and replace the fixed `rim_target = 0.985` annulus with a thin band at `nz ≈ RIM_NZ` weighted by the upper-left key; add no specular term
- [ ] 1.3 Sample the honeycomb in arc length using `select(1.0, asin(clamp(r,0.0,1.0)) / max(r,1e-4), r > 1e-4)` and re-express the ripple phase in the same arc coordinate; keep the existing wireframe intensity and flicker behavior
- [ ] 1.4 Add the interior body tint `pow(max(nz,0.0),3.0) * BODY_STRENGTH` to the circle branch's combined intensity and fold it into the existing alpha path (no `AlphaMode2d::Add`)
- [ ] 1.5 Replace the hard `if r > 1.0 { discard; }` with an `fwidth`-derived antialiased silhouette falloff consistent with `overdrive_field.wgsl`
- [ ] 1.6 Verify the plane branch (`shape_type >= 0.5`), the `VertexOutput` struct, `ShieldMaterial`, `alpha_mode()`, and every Rust file are unchanged

## 2. Documentation

- [ ] 2.1 Amend the Visuals subsection of `docs/combat.md` (lines 108-114) to describe the hemispherical reconstruction, collider-aligned thin upper-left-lit rim, arc-length honeycomb and ripple, antialiased silhouette, and the circle-only 5% interior body replacing strict 0% fill

## 3. Verification

- [ ] 3.1 Run `cargo dev` in the background, confirm the circle shield reads as a dome with a crisp upper-left-lit rim and foreshortened cells, confirm the plane shield and hull-intersection behavior are unchanged, then terminate the background process
- [ ] 3.2 Confirm visually at maximum zoom-out that honeycomb wires remain at least 2 pixels wide
- [ ] 3.3 Confirm no changes to `component.rs`, `system.rs`, `mod.rs`, `block.ron`, `constant.rs`, colliders, or `LAYER_SHIELD`, and that no RON or uniform fields were added

## Context

`shield.wgsl:38-71` shades a flat `Circle` mesh entirely in disc UV space. The boundary is a fixed annulus (`rim_target = 0.985`), the honeycomb is `hex_coords(p * 14.0)` with constant screen-space cell size, and the ripple uses `dist`. Nothing in the shader depends on view angle or reconstructed depth, so the eye reads a decal. The `VertexOutput` struct declares `world_normal` and `world_position` but the fragment shader ignores them; because the source mesh is flat, its interpolated normal is constant `(0,0,1)` and cannot supply curvature regardless.

Two recorded constraints bound the solution:

- `openspec/changes/archive/2026-11-20-render-layering-architecture/design.md:14` — multi-camera `RenderLayers` and separate pass pipelines are a non-goal.
- `openspec/specs/shield-defense/spec.md:73-78` — the barrier renders at `LAYER_SHIELD` (`Z = 160.0`) above hull, turrets, and projectiles.

The current camera already applies `Bloom` with additive compositing and `TonyMcMapface` tonemapping (`core/src/system/mod.rs:28-37`), so glow is a pipeline feature and the shader does not need to fake it with a wide gradient or additive blend. The updated `AGENTS.md` art style additionally forbids cartoon-like results, unnecessary gradients, and large highlights, and requires one consistent upper-left light direction.

## Goals / Non-Goals

**Goals:**

- Make circle shields read as a hemispherical dome while keeping the flat `Mesh2d` pipeline intact.
- Derive the rim from reconstructed surface depth and keep it aligned with the physics collider radius.
- Foreshorten the honeycomb toward the silhouette without adding geometry.
- Apply one consistent upper-left light cue with no specular lobe.
- Add a restrained interior body tint that preserves the dark, grounded palette.

**Non-Goals:**

- No true 3D mesh, 3D camera, `RenderLayers`, depth buffer, or second pass.
- No front/back hemisphere split, hull-contact glow, or depth-aware occlusion of the ship.
- No changes to the plane shield, colliders, physics, lifecycle, power, config, or layer constants.
- No new RON knobs, uniform fields, or Rust code.

## Decisions

### 1. Reconstruct the hemisphere from the disc UV

With `p = (uv - 0.5) * 2.0` and `r = length(p)`, the orthographic projection of a unit sphere gives the visible-hemisphere normal:

```wgsl
let nz = sqrt(max(0.0, 1.0 - r * r));
let n = vec3<f32>(p, nz);
```

No vertex shader, mesh, or `world_normal` varying is required. This is the single source of all new curvature cues.

**Rationale**: the smallest change that introduces real curvature information while satisfying the archived multi-camera non-goal.

### 2. Thin depth-derived rim, matched to the collider

The current rim sits at `r = 0.985`. Since `nz = sqrt(1 - r²)`, that radius is `nz ≈ 0.1726`. Define `RIM_NZ = 0.173` and a narrow band:

```wgsl
let key = 0.65 + 0.35 * max(dot(n.xy, KEY_LIGHT.xy), 0.0);
let rim_band = smoothstep(0.012, 0.0, abs(nz - RIM_NZ)) * key;
```

`KEY_LIGHT = (-0.477, 0.667, 0.572)` is pre-normalized and points upper-left, matching the art-style lighting rule. No `pow()` fresnel wash and no specular term: the key modulates only the thin rim, so there is no broad gradient and no highlight blob.

**Rationale**: preserves the "rim aligned with physics boundary" requirement in `docs/combat.md:111` exactly, while replacing the flat annulus with a curvature-derived one. The old broad specular lobe was computed at roughly 178px wide on a radius-1028 shield and was rejected as cartoon-like.

### 3. Honeycomb in arc-length space

Uniform screen-space cells are the strongest "flat decal" tell. Sample the grid in surface arc length instead:

```wgsl
let arc = select(1.0, asin(clamp(r, 0.0, 1.0)) / max(r, 1e-4), r > 1e-4);
let hex = hex_coords(p * arc * HEX_DENSITY);
```

`|p * arc| = asin(r)` runs from 0 to π/2; sampling a uniform grid there makes cells compress radially near the silhouette. The `select` and `max(r, 1e-4)` guard prevent division by zero at the center. The ripple phase uses the same arc coordinate so it travels at constant surface speed. Cell interiors keep the thin wireframe only — the `hex.xy` values are not used for bevel shading.

**Rationale**: delivers the main 3D read with no geometry and no new per-cell lighting, consistent with "geometric seams" and "avoid unnecessary gradients."

### 4. Restrained interior body

```wgsl
let body = pow(max(nz, 0.0), 3.0) * 0.05;
let combined = max(rim_band, hex_intensity) + body;
```

The tint peaks at the dome center and vanishes at the rim. With `AlphaMode2d::Blend` it darkens/tints the hull behind rather than adding light, so it does not fight the restrained palette or existing bloom.

**Rationale**: chosen to give the bubble volume while staying far below a visible wash. This deliberately supersedes the old "0% background fill" rule for circle shields only, and is recorded in `docs/combat.md`.

### 5. Constants stay in WGSL for now

`HEX_DENSITY`, `RIM_NZ`, `RIM_WIDTH`, `KEY_LIGHT`, `KEY_FLOOR`, `KEY_RANGE`, `BODY_STRENGTH`, and `SPEC`-free ripple constants are file-level `const` values. `ShieldMaterial` and all `.ron` files are untouched.

**Rationale / deviation**: `AGENTS.md` prefers data-driven values loaded from `.ron`, but shader look constants are fastest to iterate via WGSL hot reload, and adding uniform fields would require lockstep edits to both the Rust `AsBindGroup` struct and the WGSL struct plus a recompile. This is an accepted temporary deviation, not a permanent design. If tuning demand appears, promote the constants into vec4-aligned uniform blocks and a `.ron` `shield_visual` section in a later change.

### 6. Keep `Blend`; do not touch the plane branch

`alpha_mode()` is a method on the shared `ShieldMaterial`, so `AlphaMode2d::Add` would change the plane shield too. Bevy's additive blend ignores the written alpha for RGB, which would silently disable the existing `hp_ratio` and `flicker` fade. Bloom already supplies glow.

**Rationale**: preserves the circle-only scope and the health-driven visibility requirement, and avoids over-brightening against tonemapping.

### 7. Antialiased silhouette

Replace the hard `if r > 1.0 { discard; }` with an `fwidth`-derived alpha falloff, mirroring `overdrive_field.wgsl:25-29`, so the thin rim at `r = 0.985` has a clean edge.

## Risks / Trade-offs

- **It is still a fake**: no parallax and no hull occlusion; a ship inside the dome is still overlayed by the shield. Explicitly out of scope.
- **Pixel-density check**: at the 1920×1080 design frame, one world unit is one pixel (`docs/camera.md:5-6`); radius 1028 with density 14 gives ~147px cells, well above the 2px minimum. Must be visually confirmed at maximum zoom-out, where cells shrink with the camera scale.
- **`asin` near the rim**: guarded by `clamp(r, 0.0, 1.0)` and `max(r, 1e-4)`; without the guard the arc ratio produces `NaN` at the center.
- **Local `VertexOutput` coupling**: the hand-declared struct exists because Bevy's `Mesh2dVertexOutput` import broke previously (commit `97048d1`). This change does not alter the struct or add a vertex shader, so that coupling is unchanged.
- **Bloom interaction**: the 5% body and rim boost are tuned for the current `Bloom` intensity 0.3; in-game verification is required.
- **No tuning knobs**: accepted per decision 5; iteration happens through WGSL hot reload.

## Open Questions

None. All design decisions were resolved with the user before this proposal was created.

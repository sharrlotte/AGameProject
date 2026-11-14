# 3D Model Asset Specifications and Required Changes

## 1. Scope & Purpose

This document outlines the exact technical standards, asset defects, and necessary modifications for 3D glTF/GLB models used in the engine. It serves as the specification for 3D model authoring, Blender pipeline integration, and asset validation.

| Asset family | Location | Section |
| :--- | :--- | :--- |
| Player character | `game/assets/models/player.glb` | §2 |
| Weapons | `game/assets/models/weapons/*.glb` | §3 |
| Enemies | `game/assets/models/enemies/*.glb` | §4 |
| Materials & shading (all assets) | — | §5 |

---

## 2. Player Character Model (`game/assets/models/player.glb`)

### Current State & Observed Issues
1. **Backface Culling & Winding:**
   - Previous code used a 16-level hierarchy traversal runtime hack (`disable_player_backface_culling`) because the model's cloth/armor geometry or normals suffered from backface culling artifacts in Bevy's PBR pipeline.
   - **Required Fix:** Geometry normals must be unified and outward-facing, or materials must be explicitly exported with double-sided rendering (`doubleSided: true` in glTF material definition).
2. **Transform Safety Offsets (`config/player/visuals.ron`):**
   - The asset currently requires runtime compensations:
     - `scale: 0.37` (model was authored in arbitrary units rather than standard metric meters).
     - `y_offset: -0.9` (model origin / root bone is located at the chest/hips rather than grounded at the feet $Y=0$).
     - `yaw_offset_deg: 180.0` (model faces $+Z$ or $+Y$ at rest instead of engine forward $-Z$).
   - **Required Fix:** 
     - Re-scale the mesh and armature to true 1:1 real-world metric units (character standing approx. 1.8m tall).
     - Place the armature root origin directly between the feet on the ground plane ($Y=0$).
     - Orient the character rest pose to face forward along $-Z$, with $+Y$ as up.
     - Once fixed, `visuals.ron` can normalize to `scale: 1.0`, `y_offset: 0.0`, and `yaw_offset_deg: 0.0`.
3. **Rig / Skeleton Hierarchy:**
   - Current rig has 31 deform bones.
   - IK requires standard naming conventions for the dual-arm chain (`Shoulder.R/L`, `UpperArm.R/L`, `LowerArm.R/L`, `Palm.R/L`) and finger/hand bones (`Torso`, `MiddleHand.R/L`, `Fingers.R/L`, `Thumb1.R/L`, `Thumb2.R/L`).
   - If non-standard names are used in future armatures, they must match the configurable fields in `AnimCatalog::ik`.
4. **Locomotion Animations:**
   - The raw idle clip (`Man_Idle`) has an exaggerated spread-arm silhouette. Previously, a procedural 50% torso X-rotation hack was applied.
   - **Required Fix:** Author an ergonomic rest idle pose in the glTF animation clip itself, with arms resting naturally at the sides and matching combat grip expectations.

---

## 3. Weapon Models (`game/assets/models/weapons/*.glb`)

### Current Weapon Roster
- `m16.glb`
- `m1911.glb`
- `mp5.glb`
- `mp7.glb`
- `scar.glb`
- `rifle.glb` (reference prototype)

### Asset Requirements & Sockets Contract
Every weapon model must adhere to the standardized socket and coordinate hierarchy:

| Socket Node | Role | Required Placement / Behavior |
| :--- | :--- | :--- |
| `Grip` | Primary right-hand IK attachment | Placed at the trigger grip where the right palm rests. The origin $(0,0,0)$ of the weapon space should coincide with `Grip`. |
| `Foregrip` | Support left-hand IK attachment | Placed along the barrel/handguard where the support hand grips. Must be separated from `Grip` by at least 8cm ($>0.08$m). |
| `Magazine` | Left-hand target during reload & detachable mag mesh | Placed at the magazine well. The magazine geometry should be an independent child node of this socket so runtime reload animation can displace it. |
| `Muzzle` | Projectile spawn origin & muzzle flash VFX | Placed at the tip of the barrel pointing along $-Z$. |

### Coordinate Orientation & Normalization
1. **Weapon Space Axes:**
   - Standard engine weapon space is:
     - **Forward:** $-Z$
     - **Up:** $+Y$
     - **Right:** $+X$
2. **Current Workaround (Model Spaces):**
   - Converted assets (`m16`, `m1911`, `mp5`, `scar`) were exported with Blender/glTF axis disparities (`ForwardNegYUpZ`: $-Y$ forward, $+Z$ up).
   - `m16.glb` additionally required a $180^\circ$ model rotation (`model_rotation_deg: (0.0, 180.0, 0.0)`) and flipped grip vectors because the barrel faced $+Z$.
   - **Required Fix:** Re-export all weapon models natively in engine space ($-Z$ forward, $+Y$ up, scale 1.0, grip at origin). This will allow removing `model_space: ForwardNegYUpZ`, `model_rotation_deg`, and `model_scale` adjustments in `weapon.ron`.
3. **Baked vs Override Sockets:**
   - The engine supports `socket_overrides` in `weapon.ron` to adjust socket positions when models lack baked nodes.
   - **Required Fix:** Bake explicit empty/locator nodes for `Grip`, `Foregrip`, `Magazine`, and `Muzzle` directly into the `.glb` files with correct rest transforms, eliminating the need for `socket_overrides`.

---

## 4. Enemy Models (`game/assets/models/enemies/*.glb`)

### 4.1 Scope & Delivery Priority

Seven enemy archetypes are specified here. Player and weapon assets are covered above and must not be mixed into this section.

| Archetype | Runtime today | Priority | Delivery note |
| :--- | :--- | :---: | :--- |
| **Tank** | Primitive assembly planned | 1 | First GLB target |
| **Walker** | Primitive capsule | 2 | Replaces the primitive |
| **Runner** | Primitive capsule | 2 | Replaces the primitive |
| **Spitter** | Not implemented | 3 | Model may be prepared before its behavior lands |
| **Exploder** | Not implemented | 3 | Model may be prepared before its behavior lands |
| **Screamer** | Not implemented | 3 | Requires a jaw bone |
| **Stalker** | Not implemented | 3 | Requires a dedicated cloak material slot |

Each archetype keeps its primitive runtime fallback until its GLB is delivered, validated, and approved. Gameplay work must not block on a model.

### 4.2 Shared Export Contract

The normalization rules in §2.2 apply unchanged to every enemy: one Blender unit equals one meter, the GLB uses $+Y$ up and $-Z$ forward, the origin sits between the feet at $(0,0,0)$ with feet resting at $Y=0$, the mesh is centered on $X=0$, export scale is `1.0`, and no runtime scale, yaw, or offset correction is allowed.

Enemy-specific requirements:

- One GLB per archetype: `walker.glb`, `runner.glb`, `tank.glb`, `spitter.glb`, `exploder.glb`, `screamer.glb`, `stalker.glb` under `game/assets/models/enemies/`.
- Each GLB contains its mesh, armature, animation clips, materials, and socket nodes in a single file.
- Source `.blend` files live under `tools/blender/enemies/` and are never shipped in `game/assets/`.
- Mesh node name is `<Kind>_Mesh` (for example `Tank_Mesh`); armature node name is `<Kind>_Rig` (for example `Tank_Rig`).
- The export contains no cameras, lights, colliders, gameplay markers, or unneeded empties.

### 4.3 Style, Materials, and Skinning

- Style is low-poly and stylized with flat color blocks and a strong silhouette that reads at the 20–30 m top-down camera distance.
- Each archetype uses 2–4 materials. Textures are omitted; a shared palette atlas requires approval before modeling.
- Material names follow `<kind>_<part>` (for example `tank_steel`, `spitter_sac`).
- Base roughness stays between `0.6` and `0.9`. Metallic values above `0.2` are reserved for Tank steel.
- Base colors are mid-tone so the runtime white hit flash reads clearly.
- Baked lighting, baked ambient occlusion, and baked highlights must not be included in base color.
- Alpha blending is reserved for the Stalker cloak material slot so runtime alpha can change without fading the body or eyes. Every other material is opaque.
- Skinning uses at most four bone influences per vertex. No unweighted vertices and no shape keys.
- Normals, tangents, double-sided flags, and PBR material rules are defined once in §5 and apply unchanged.

### 4.4 Rig Contract

Every enemy uses the shared humanoid chain below. Bone names are player-compatible so future Mixamo clips can be retargeted without renaming.

```
Root (non-deform)
└─ Hips
   ├─ Abdomen
   │  └─ Torso
   │     ├─ Neck
   │     │  └─ Head
   │     │     └─ Jaw            (Screamer required; optional elsewhere)
   │     ├─ Shoulder.L → UpperArm.L → LowerArm.L → Palm.L
   │     └─ Shoulder.R → UpperArm.R → LowerArm.R → Palm.R
   ├─ UpperLeg.L → LowerLeg.L → Foot.L
   └─ UpperLeg.R → LowerLeg.R → Foot.R
```

- The standard chain is 21 bones including `Root` and `Jaw`.
- Deform bones must not exceed 24 per archetype.
- Bone names match the tree exactly, including the `.L` / `.R` suffixes.
- IK controls, constraints, and drivers are not exported; bake all animation to the deform bones.
- Extra bones are allowed only for:
  - Spitter sacs: up to 3 bones (for example `Sac.01`–`Sac.03`), otherwise parent sacs to `Torso`.
  - Stalker cloak: up to 2 bones, otherwise parent the cloak to `Hips` or `Torso`.
- Bone roll is consistent across the left and right chains. Negative scale is forbidden.
- `Root` stays at the origin; gameplay moves the entity, not the animation.

### 4.5 Animation Contract

- Clips are authored at 30 fps and are in place.
- Root translation is zero for every clip. Vertical hip bob is allowed.
- Clip names are exactly `idle`, `walk`, `run`, `windup`, `attack`, `stagger`, `death`, plus the archetype extras `spit`, `scream`, `explode`, and `cloak` where listed.
- If the exporter prefixes the armature name, the suffix after `|` must still be the exact clip name.
- `idle`, `walk`, and `run` loop. All other clips are one-shots.
- `windup` ends at the moment the attack is committed; `attack` begins at impact.
- `death` ends in a settled pose and never loops.
- The simulation drives timing; clips contain no gameplay events, root motion, or camera motion.

| Clip | Duration | Loop | Used by |
| :--- | ---: | :--- | :--- |
| `idle` | 2.0 s | Yes | All |
| `walk` | 1.0 s | Yes | All |
| `run` | 0.65 s | Yes | Runner, Stalker |
| `windup` | 0.45 s | No | Tank |
| `attack` | 0.35 s | No | Tank, Walker, Runner, Stalker |
| `stagger` | 0.35 s | No | All |
| `death` | 1.2 s | No | All except Tank |
| `death` | 1.4 s | No | Tank |
| `spit` | 0.5 s | No | Spitter |
| `scream` | 1.0 s | No | Screamer |
| `explode` | 0.6 s | No | Exploder |
| `cloak` | 0.4 s | No | Stalker |

### 4.6 Sockets and Markers

Sockets are empty nodes in the exported GLB, not bones. Names are unique.

| Socket | Archetype | Placement | Fallback |
| :--- | :--- | :--- | :--- |
| `Muzzle` | Spitter | Center of the mouth, facing `-Z` | `Head` origin |
| `BlastOrigin` | Exploder | Chest center, inside the pustule mass | `Torso` origin |
| `ScreamOrigin` | Screamer | Throat or open mouth center | `Head` origin |
| `WeakPoint` | Tank | Upper-back center, on the exposed back | `Torso` origin |

Sockets are authored in model space. Runtime falls back safely when a socket is missing and logs one warning per missing socket.

### 4.7 Collision Fit and Triangle Budgets

The collider is a vertical capsule centered at $Y = radius + half\_height$. The visual origin stays at the feet. Runtime spawns each archetype at its own collider center and offsets the visual child so the model's feet land at world $Y = 0$.

| Archetype | Visual height | Visual width | Collider radius | Collider half-height | Capsule total height | Triangle budget |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| Walker | 1.80 m | 0.55 m | 0.45 m | 0.45 m | 1.80 m | 1,200 |
| Runner | 1.55 m | 0.45 m | 0.35 m | 0.40 m | 1.50 m | 900 |
| Tank | 2.80 m | 1.90 m | 0.75 m | 0.70 m | 2.90 m | 2,500 |
| Spitter | 1.90 m | 0.70 m | 0.45 m | 0.50 m | 1.90 m | 1,600 |
| Exploder | 1.80 m | 1.20 m | 0.55 m | 0.35 m | 1.80 m | 1,800 |
| Screamer | 1.90 m | 0.55 m | 0.40 m | 0.55 m | 1.90 m | 1,400 |
| Stalker | 1.70 m | 0.60 m | 0.40 m | 0.45 m | 1.70 m | 1,300 |

- The mesh may sit slightly inside the collider, but no body part may visibly float above the floor or clip through the capsule center.
- Width includes shoulders, sacs, and pustules; those parts may extend slightly beyond the capsule but must not change the collider size.
- Parts that extend above the collider (Tank shoulders, Screamer head) are visual only.
- The Tank is intentionally taller than the 2.6 m arena walls; this is a readability choice, not a collision bug.

### 4.8 Export Validation

Every enemy GLB is exported as binary glTF with $+Y$ up conversion enabled, modifiers applied, armature and skinning included, animation clips included, one skin per archetype, and no cameras, lights, or unused empties.

Before a GLB is accepted, the exporter must report and verify:

1. Forward axis is `-Z`, up is `+Y`, and scale is `1.0`.
2. Origin is between the feet and feet rest at `Y = 0`.
3. Triangle count is within the archetype budget.
4. Bone count is at most 24 and bone names match the shared chain.
5. Required clip names exist and loop flags match §4.5.
6. Required sockets exist with unique names.
7. Material count is 2–4 and material names follow `<kind>_<part>`.
8. There is no root motion in any clip.
9. There are no cameras, lights, colliders, or gameplay components in the export.

### 4.9 Preview and Approval

Mesh, material, rig, socket, and clip changes are previewed before the final GLB is written. Each preview set includes:

- Front, left, back, and three-quarter renders.
- Rest pose and attack pose.
- One render with the player model for scale.
- One render at the game camera distance for silhouette readability.

No GLB is written until the project owner approves the previews.

### 4.10 Runtime Integration Contract

- The GLB is mounted under a `VisualBody` root that rotates in yaw so the model's `-Z` forward points along the enemy facing direction.
- The enemy simulation entity sits at the collider center. The visual child is offset downward by $radius + half\_height$ so the GLB's feet land at world $Y = 0$.
- The runtime must not apply scale, yaw, or Y-offset corrections to enemy models.
- Hit flash covers every mesh part, and each part's base material handle is cached so the flash can restore it.
- The Stalker cloak fades only its cloak material slot; body and eyes stay visible.
- A missing GLB or missing socket degrades to the primitive fallback with a warning, never a panic.

### 4.11 Tank

**Role:** Armored pressure unit. It walks through basic fire and forces the player to flank or use heavy weapons.

```
FRONT                         BACK
    [ small head ]                [ small head ]
 [pauldron][pauldron]          [shoulder][shoulder]
 [ bolted front plate ]        [ exposed back / weak point ]
    [ heavy belly ]               [ belt ]
   [ boot ] [ boot ]             [ boot ] [ boot ]
```

**Silhouette:** Broad trapezoid torso, small head sunk between shoulders, thick arms, heavy boots. The front plate is the brightest large shape; the back is visibly unprotected.

**Parts:**
- Torso under-suit with a bolted front plate covering chest and abdomen.
- Two shoulder pauldrons, one slightly asymmetric.
- Small head with a respirator or half-mask.
- Thick upper arms and forearms; fists large enough to read at camera distance.
- Heavy boots and knee pads.
- Exposed upper-back weak point with straps, torn suit, or pale scar tissue.

**Materials:**

| Material | Color | Metallic | Roughness | Notes |
| :--- | :--- | ---: | ---: | :--- |
| `tank_steel` | `#3E4650` | 0.65 | 0.45 | Front plate, pauldrons, boot caps |
| `tank_rust` | `#8A5A32` | 0.10 | 0.80 | Plate edges, bolts, straps |
| `tank_suit` | `#24272C` | 0.00 | 0.90 | Under-suit, joints, boots |
| `tank_weak` | `#7A3B32` | 0.00 | 0.70 | Exposed back weak point |

**Rig:** Standard 21-bone chain. Pauldrons parented to `Shoulder.L/R`; front plate parented to `Torso`. `WeakPoint` socket at the upper-back center.

**Clips:** `idle`, `walk`, `windup` 0.45 s, `attack` 0.35 s, `stagger`, `death` 1.4 s.

**Gameplay tells:** The wind-up raises both arms and roots the body; the front plate is a clear armor read; the back weak point is visible when the Tank turns away.

**Triangle budget:** 2,500.

### 4.12 Walker

**Role:** Slow resource and space pressure. Dangerous in packs, not individually.

```
      [ tilted head ]
   [ uneven shoulders ]
    [ torn shirt torso ]
     [ dangling arms ]
      [ bent knees ]
     [ worn boots ]
```

**Silhouette:** Hunched forward, one shoulder higher, long dangling arms, bent knees, head tilted.

**Parts:**
- Torso in a torn work shirt with a visible rib or shoulder wound.
- Slack jaw and uneven hair.
- One sleeve torn, exposing a long forearm.
- Work pants; one boot missing.
- Slight asymmetry in arm length and shoulder height.

**Materials:**

| Material | Color | Metallic | Roughness | Notes |
| :--- | :--- | ---: | ---: | :--- |
| `walker_skin` | `#6E8B5A` | 0.00 | 0.80 | Desaturated green-gray flesh |
| `walker_shirt` | `#4C5560` | 0.00 | 0.85 | Faded blue-gray work shirt |
| `walker_pants` | `#3B3A36` | 0.00 | 0.90 | Dark brown-gray pants |

**Rig:** Standard chain; `Jaw` optional. No extra bones.

**Clips:** `idle`, `walk`, `attack`, `stagger`, `death`.

**Gameplay tells:** Uneven shamble and head tilt distinguish it from the Runner at range.

**Triangle budget:** 1,200.

### 4.13 Runner

**Role:** Fast reaction check. Punishes slow reloads and tunnel vision.

```
        [ head, wide mouth ]
      [ narrow leaning torso ]
      [ long arm ] [ long arm ]
        [ short hips ]
       [ bare feet ]
```

**Silhouette:** Lean, forward-leaning, long arms, short torso, bare feet, elbows held wide.

**Parts:**
- Narrow torso in a torn tank top.
- Long forearms and oversized hands.
- Short hips and long lower legs.
- Bare feet with visible tendons.
- Wide mouth and sunken eyes.

**Materials:**

| Material | Color | Metallic | Roughness | Notes |
| :--- | :--- | ---: | ---: | :--- |
| `runner_skin` | `#C24A38` | 0.00 | 0.75 | Inflamed red flesh |
| `runner_cloth` | `#6B2F2A` | 0.00 | 0.85 | Dark torn tank top |
| `runner_eye` | `#FFE08A` | 0.00 | 0.30 | Optional emissive eyes |

**Rig:** Standard chain; `Jaw` optional. No extra bones.

**Clips:** `idle`, `walk`, `run`, `attack`, `stagger`, `death`.

**Gameplay tells:** Low forward lean and wide elbows read as a sprinter; the lunge starts from `attack`.

**Triangle budget:** 900.

### 4.14 Spitter

**Role:** Ranged area denial. Forces the player out of static camping spots and threatens turrets.

```
        [ open jaw ]
     [ thin neck ]
  [ 3-5 glowing sacs ]
   [ thin arms hanging ]
      [ thin legs ]
```

**Silhouette:** Hunched, thin limbs, bulbous glowing chest and throat sacs, open jaw, acid drool.

**Parts:**
- Thin torso with 3–5 visible sacs clustered at the throat and upper chest.
- Open jaw with a dark throat.
- Drooping thin arms with long fingers.
- Thin legs and backward knees.
- Rag wrap at the waist.

**Materials:**

| Material | Color | Metallic | Roughness | Notes |
| :--- | :--- | ---: | ---: | :--- |
| `spitter_skin` | `#8A6B4A` | 0.00 | 0.80 | Sickly brown flesh |
| `spitter_sac` | `#B6FF4A` | 0.00 | 0.25 | Emissive strength 2.0 |
| `spitter_rag` | `#4A4438` | 0.00 | 0.90 | Dirty cloth wrap |

**Rig:** Standard chain. Up to three optional sac bones; otherwise parent sacs to `Torso`. `Jaw` required for the spit pose. `Muzzle` socket at the mouth center.

**Clips:** `idle`, `walk`, `spit`, `stagger`, `death`.

**Gameplay tells:** Sacs pulse and brighten before `spit`; the rear-back pose telegraphs the arc.

**Triangle budget:** 1,600.

### 4.15 Exploder

**Role:** Spacing and trigger-discipline check. Punishes spraying in tight spaces.

```
       [ sunken head ]
    [ short arm ] [ short arm ]
    [ bloated pustule belly ]
       [ short legs ]
```

**Silhouette:** Round, bloated belly, short limbs, head sunk into the torso, pustules breaking the outline.

**Parts:**
- Distended torso with 5–8 raised pustules.
- Cracked skin between pustules.
- Short thick arms and legs.
- Small head with a pained expression.
- Torn waistband.

**Materials:**

| Material | Color | Metallic | Roughness | Notes |
| :--- | :--- | ---: | ---: | :--- |
| `exploder_skin` | `#A85A3C` | 0.00 | 0.80 | Burned orange-brown flesh |
| `exploder_pustule` | `#FF7A2E` | 0.00 | 0.20 | Emissive strength 2.5 |
| `exploder_crack` | `#3A1E16` | 0.00 | 0.90 | Dark cracks and seams |

**Rig:** Standard chain. Pustules parented to `Torso`; no extra bones required. `BlastOrigin` socket at chest center.

**Clips:** `idle`, `walk`, `stagger`, `explode`, `death`.

**Gameplay tells:** Pustules pulse faster as the player approaches and flare before `explode`.

**Triangle budget:** 1,800.

### 4.16 Screamer

**Role:** Priority target. If it finishes its scream, the room becomes a swarm.

```
       [ oversized head ]
       [ huge open jaw ]
     [ long thin neck ]
   [ gaunt torso, gown ]
   [ long arms, long fingers ]
       [ thin legs ]
```

**Silhouette:** Tall, gaunt, oversized head and jaw, long fingers, tattered gown, pale skin.

**Parts:**
- Emaciated torso with visible ribs.
- Long neck and oversized head.
- Hinged jaw that opens wider than a human jaw.
- Long arms with elongated fingers.
- Tattered gown or hospital wrap.

**Materials:**

| Material | Color | Metallic | Roughness | Notes |
| :--- | :--- | ---: | ---: | :--- |
| `screamer_skin` | `#B9B3A6` | 0.00 | 0.75 | Pale gray flesh |
| `screamer_throat` | `#D8C7FF` | 0.00 | 0.30 | Emissive strength 1.8 |
| `screamer_gown` | `#5A5560` | 0.00 | 0.90 | Gray tattered gown |

**Rig:** Standard chain. `Jaw` bone is required for the scream pose. `ScreamOrigin` socket at the throat or mouth center.

**Clips:** `idle`, `walk`, `scream`, `stagger`, `death`.

**Gameplay tells:** Jaw opens and throat glows during the scream wind-up; the head is the readable priority target.

**Triangle budget:** 1,400.

### 4.17 Stalker

**Role:** Awareness and flanking check. Ambushes players who only watch the front.

```
      [ low head, glowing eyes ]
   [ spiny back ] [ spiny back ]
   [ long arms with claws ]
   [ cloak membrane behind limbs ]
      [ crouched legs ]
```

**Silhouette:** Low crouch, elongated limbs, spiny back, smooth dark carapace, membrane cloak.

**Parts:**
- Low, forward-crouched torso.
- Elongated arms with three-claw hands.
- Spiny back plates.
- A separate cloak membrane attached at the shoulders and hips.
- Smooth head with two glowing eyes and no visible mouth.

**Materials:**

| Material | Color | Metallic | Roughness | Notes |
| :--- | :--- | ---: | ---: | :--- |
| `stalker_carapace` | `#2E3038` | 0.10 | 0.55 | Dark blue-black carapace |
| `stalker_cloak` | `#1B1D24` | 0.00 | 0.80 | Alpha blend; runtime fades this slot |
| `stalker_eye` | `#8AF0FF` | 0.00 | 0.20 | Emissive strength 3.0 |

**Rig:** Standard chain, with the crouch authored into the rest pose. Up to two optional cloak bones; otherwise parent the cloak to `Hips` or `Torso`. The cloak must use its own material slot.

**Clips:** `idle`, `walk`, `run`, `cloak`, `attack`, `stagger`, `death`.

**Gameplay tells:** The cloak fades toward transparent while the eyes stay visible; `cloak` plays on decloak before the pounce.

**Triangle budget:** 1,300.

### 4.18 Palette Summary

| Token | Hex | Used by |
| :--- | :--- | :--- |
| `walker_skin` | `#6E8B5A` | Walker flesh |
| `walker_shirt` | `#4C5560` | Walker shirt |
| `walker_pants` | `#3B3A36` | Walker pants |
| `runner_skin` | `#C24A38` | Runner flesh |
| `runner_cloth` | `#6B2F2A` | Runner top |
| `runner_eye` | `#FFE08A` | Runner eyes |
| `tank_steel` | `#3E4650` | Tank plate |
| `tank_rust` | `#8A5A32` | Tank trim |
| `tank_suit` | `#24272C` | Tank under-suit |
| `tank_weak` | `#7A3B32` | Tank weak point |
| `spitter_skin` | `#8A6B4A` | Spitter flesh |
| `spitter_sac` | `#B6FF4A` | Spitter sacs |
| `spitter_rag` | `#4A4438` | Spitter cloth |
| `exploder_skin` | `#A85A3C` | Exploder flesh |
| `exploder_pustule` | `#FF7A2E` | Exploder pustules |
| `exploder_crack` | `#3A1E16` | Exploder cracks |
| `screamer_skin` | `#B9B3A6` | Screamer flesh |
| `screamer_throat` | `#D8C7FF` | Screamer throat |
| `screamer_gown` | `#5A5560` | Screamer gown |
| `stalker_carapace` | `#2E3038` | Stalker body |
| `stalker_cloak` | `#1B1D24` | Stalker cloak |
| `stalker_eye` | `#8AF0FF` | Stalker eyes |

### 4.19 Animation Matrix

| Clip | Walker | Runner | Tank | Spitter | Exploder | Screamer | Stalker |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `idle` | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| `walk` | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| `run` | — | Yes | — | — | — | — | Yes |
| `windup` | — | — | Yes | — | — | — | — |
| `attack` | Yes | Yes | Yes | — | — | — | Yes |
| `stagger` | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| `death` | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| `spit` | — | — | — | Yes | — | — | — |
| `scream` | — | — | — | — | — | Yes | — |
| `explode` | — | — | — | — | Yes | — | — |
| `cloak` | — | — | — | — | — | — | Yes |

### 4.20 Export Checklist

- [ ] Model is authored at `1.0` scale with feet at `Y = 0` and origin between the feet.
- [ ] Exported GLB faces `-Z` with `+Y` up.
- [ ] Mesh and armature use `<Kind>_Mesh` and `<Kind>_Rig` names.
- [ ] Bone chain matches §4.4 and stays at or below 24 bones.
- [ ] All required clip names exist with correct loop flags and no root motion.
- [ ] Required sockets exist with unique names.
- [ ] Materials use `<kind>_<part>` names and stay within the 2–4 material budget.
- [ ] Triangle count is inside the archetype budget.
- [ ] No cameras, lights, colliders, or unused empties are exported.
- [ ] §5 material, normal, tangent, and double-sided rules are satisfied.
- [ ] Front, left, back, three-quarter, rest, attack, player-scale, and camera-distance previews are rendered.
- [ ] Previews are approved before the final GLB is written.
- [ ] The GLB is inspected and its triangle, node, bone, and clip counts are reported.

### 4.21 Open Decisions

- Whether enemy GLBs replace the primitive assemblies archetype-by-archetype or land as one visual pass.
- Whether the Stalker cloak fades as one material slot or uses a second translucent shell mesh.
- Whether the Tank back weak point later becomes a real hit zone instead of a visual/socket marker.
- Whether enemy clips carry an armature prefix in the GLB or are renamed after export.
- Final source `.blend` location under `tools/blender/enemies/`.

---

## 5. Visual Materials & Shaders

1. **Double-Sided Flags:**
   - Ensure all thin materials (straps, weapon sling mounts, cloth edges) have `doubleSided: true` enabled in the glTF exporter settings.
2. **Normals & Tangents:**
   - Export vertex tangents with models to avoid shading artifacts under normal mapping.
   - Recalculate normals inside Blender (`Shift + N`) to prevent inverted face lighting in Bevy.
3. **PBR Standard:**
   - Materials must adhere to standard metallic-roughness PBR workflows without custom unsupported Blender shader node networks.

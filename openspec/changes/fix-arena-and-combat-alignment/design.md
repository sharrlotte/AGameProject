## Context

Following live inspection via Bevy Remote Protocol (BRP) MCP in `Scene::Arena`:
1. The survivor was found at $Y = -20.0$ instead of normal ground elevation, caused by Blender Z-up exported coordinates in `tools/build_district_map.py` where $Z = -20$ mapped to glTF $Y = -20$. `reposition_player_to_spawn_marker` also overwrote the survivor's vertical translation directly without anchoring to the player's configured elevation.
2. In `process_weapon_fire`, `muzzle_pos` replaced $Y$ with absolute `player_cfg.aim_height` instead of adding it relative to the survivor's current world position (`transform.translation.y + player_cfg.aim_height`), causing bullets to spawn high in the air at screen center.
3. In `inspect_map_scene_nodes`, `Trigger_Interior_*` entities were tagged with `Sensor` and `InteriorVolume` but lacked any `Collider` or `ActiveEvents::COLLISION_EVENTS`. Rapier cannot fire collision events for collider-less sensors or pairs without active collision event flags. Additionally, exterior building walls blocking the angled camera line of sight lacked camera-to-player raycast fading.

## Goals / Non-Goals

**Goals:**
- Correct coordinate generation in `tools/build_district_map.py` (Blender $+Z$ as vertical height) and re-export `district_01.glb`.
- Ensure `reposition_player_to_spawn_marker` respects player configuration elevation (`cfg.aim_height`).
- Calculate weapon muzzle position relative to the survivor's current world position in simulation space: `Vec3::new(transform.translation.x, transform.translation.y + player_cfg.aim_height, transform.translation.z) + forward * weapon.muzzle_offset`.
- Automatically attach a Rapier `Collider` (trimesh from mesh) and `ActiveEvents::COLLISION_EVENTS` to all `Trigger_Interior_*` entities.
- Implement camera-to-player raycast line-of-sight occlusion fading for exterior walls and obstacles when standing behind them.

**Non-Goals:**
- No visual bone socket queries inside `combat` simulation (pure simulation separation requested and approved).
- No procedural floor generation or dynamic destruction.

## Decisions

### 1. Pure Simulation Relative Muzzle Position
- **Decision:** Keep `combat` pure simulation without presentation dependencies. Compute muzzle position as:
  ```rust
  let muzzle_pos = Vec3::new(
      transform.translation.x,
      transform.translation.y + player_cfg.aim_height,
      transform.translation.z,
  ) + forward * weapon.muzzle_offset;
  ```
- **Rationale:** Preserves strict architectural separation between `engine::combat` and `engine::presentation`. Corrects the absolute-Y screen center detachment immediately across all elevation states.

### 2. Auto-Collider & ActiveEvents for Interior Triggers
- **Decision:** In `inspect_map_scene_nodes`, when encountering `Trigger_Interior_*`:
  - Inspect `mesh_handle` and attach `Collider::from_bevy_mesh` (or trimesh when available, with pending mesh fallback).
  - Insert `ActiveEvents::COLLISION_EVENTS`.
  - Set `Visibility::Hidden` so trigger proxy geometry does not render.
- **Rationale:** Rapier requires both a valid `Collider` and `ActiveEvents::COLLISION_EVENTS` on at least one collider in an intersecting pair to emit `CollisionEvent::Started` and `CollisionEvent::Stopped`.

### 3. Camera Line-of-Sight Occlusion Raycast
- **Decision:** In `presentation/camera/system.rs` (or `presentation/scene/arena.rs`), cast a Rapier ray from `MainCamera` translation to the survivor's chest focus point.
  - Entities hit that are marked with `Obstacle` or `RoofOcclusion` and have a `MeshMaterial3d` or `Visibility` component receive temporary transparency/occlusion hiding.
  - When the ray no longer hits them, their original visibility/material state is restored.
- **Rationale:** Angled 55°-65° top-down perspective frequently obscures the player behind tall exterior building walls even before entering interior sensor volumes.

## Risks / Trade-offs

- **[Raycast against multiple overlapping obstacles]** → Ray could hit non-occluding props.
  - *Mitigation:* Filter raycast to `GROUP_WORLD` obstacles that lie in front of the player along the camera view vector.
- **[Asynchronous glTF mesh loading for triggers]** → The mesh for `Trigger_Interior_*` might load on a subsequent frame.
  - *Mitigation:* Add `PendingTriggerMesh` or reuse `generate_pending_mesh_colliders` so colliders attach as soon as the mesh asset is ready.

## Why

Live BRP MCP debugging identified three critical gameplay alignment defects in `Scene::Arena`:
1. The survivor entity is positioned 20m below the ground plane due to Blender Z-up to glTF Y-up axis reversal in `tools/build_district_map.py` and unchecked transform assignment in `reposition_player_to_spawn_marker`, causing the camera to look at the ground while the player model is shifted off-center.
2. Fired bullets spawn at the absolute center of the camera screen (`Y = 0.8`) instead of the survivor's current elevation and horizontal aim offset.
3. Interior roof/upper-wall occlusion triggers fail to hide roofs because `Trigger_Interior_*` nodes lack Rapier colliders and `ActiveEvents::COLLISION_EVENTS`, while exterior walls lack camera raycast line-of-sight fading when occluding the player.

Fixing these ensures accurate player centering, proper projectile simulation origin, and functional interior and exterior occlusion cutaways.

## What Changes

- **Blender District Map Regeneration:** Correct coordinate mapping in `tools/build_district_map.py` (Blender +Z as vertical elevation) and re-export `district_01.glb` so all markers, colliders, triggers, and buildings sit on the ground plane ($Y \ge 0$).
- **Survivor Spawn Height Clamping:** Update `reposition_player_to_spawn_marker` to position the survivor at the spawn marker's horizontal translation while respecting player height configuration (`aim_height`).
- **Simulation Bullet Muzzle Origin:** Correct `process_weapon_fire` to compute `muzzle_pos` relative to the survivor's actual world position (`transform.translation.y + player_cfg.aim_height`) without hardcoding absolute Y coordinates.
- **Interior Trigger Colliders & Events:** Update `inspect_map_scene_nodes` to attach a Rapier `Collider` and `ActiveEvents::COLLISION_EVENTS` to all `Trigger_Interior_*` entities so overlap events reliably trigger `ActiveInteriors`.
- **Exterior Wall Camera Occlusion:** Add camera-to-player raycast occlusion fading for exterior walls and roof obstacles that block line-of-sight to the survivor from the angled camera.

## Capabilities

### Modified Capabilities
- `handcrafted-map`: Map scene node inspection attaches valid Rapier colliders and active collision events to interior triggers; survivor positioning preserves aim height; camera raycast detects and fades exterior obstacles occluding the player.
- `weapon-roster`: Weapon projectile spawn origin computes muzzle position relative to the shooter's actual dynamic translation ($Y = \text{translation}.y + \text{aim\_height}$).

## Impact

- **Affected Code:**
  - `tools/build_district_map.py`
  - `game/assets/maps/district_01.glb`
  - `engine/src/world/arena/loader.rs`
  - `engine/src/combat/weapon/system.rs`
  - `engine/src/presentation/scene/arena.rs` or `engine/src/presentation/camera/system.rs`
- **Zero Breaking API Changes.**

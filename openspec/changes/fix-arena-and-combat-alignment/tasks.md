## 1. District Map Coordinates & Asset Generation

- [x] 1.1 Correct Blender axes in `tools/build_district_map.py` (treating +Z as vertical height and +Y as horizontal depth) for buildings, roofs, interior triggers, player spawn, enemy spawners, and barriers
- [x] 1.2 Re-generate and export `game/assets/maps/district_01.glb` using the updated Blender script

## 2. Arena Map Scene Inspection & Spawner Clamping

- [ ] 2.1 Update `inspect_map_scene_nodes` to attach `Collider`, `Sensor`, `ActiveEvents::COLLISION_EVENTS`, and `Visibility::Hidden` to `Trigger_Interior_*` entities
- [ ] 2.2 Update `generate_pending_mesh_colliders` to generate colliders for pending interior triggers
- [ ] 2.3 Update `reposition_player_to_spawn_marker` to position the survivor at marker horizontal coordinates while clamping/preserving `cfg.aim_height` vertical position

## 3. Weapon Simulation Relative Muzzle Position

- [ ] 3.1 Update `process_weapon_fire` in `engine/src/combat/weapon/system.rs` to compute `muzzle_pos` relative to `transform.translation.y + player_cfg.aim_height`
- [ ] 3.2 Update projectile firing unit tests to verify dynamic shooter elevation handling

## 4. Camera Line-of-Sight Occlusion Raycast

- [ ] 4.1 Implement `update_camera_wall_occlusion` system in `engine/src/presentation/camera/system.rs` casting a ray from the camera to the survivor and toggling visibility/transparency on occluding obstacles
- [ ] 4.2 Register `update_camera_wall_occlusion` in the presentation camera pipeline

## 5. Verification & Live Inspection

- [ ] 5.1 Run test suites (`cargo test -p engine`) to ensure all tests pass
- [ ] 5.2 Launch dev game with BRP MCP and verify survivor centering, bullet muzzle alignment, and rooftop/wall occlusion cutaway in Arena

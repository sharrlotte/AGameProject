import bpy

# Reset scene to empty
bpy.ops.wm.read_factory_settings(use_empty=True)

# Coordinate conversion note:
# Blender is Z-up: (X_blend, Y_blend, Z_blend)
# Standard glTF exporter maps:
# glTF X = Blender X
# glTF Y = Blender Z  (vertical elevation)
# glTF Z = -Blender Y (depth: -Z is North, +Z is South)

# 1. Ground plane (Visual)
bpy.ops.mesh.primitive_plane_add(size=120.0, location=(0.0, 0.0, 0.0))
ground = bpy.context.active_object
ground.name = "Visual_Ground"

# 2. Road surface (Visual, slightly above ground to prevent Z-fighting)
bpy.ops.mesh.primitive_plane_add(size=20.0, location=(0.0, 0.0, 0.02))
road = bpy.context.active_object
road.name = "Visual_Road_Main"
road.scale = (1.0, 5.0, 1.0)

# 3. Gas station hollow walls (Occluder_ prefix: visible + static collider + camera line-of-sight fade)
# Footprint X 3..17, glTF Z -5..5, height 5
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(3.2, 0.0, 2.5))
wall_w = bpy.context.active_object
wall_w.name = "Occluder_GasStation_Wall_West"
wall_w.scale = (0.4, 10.0, 5.0)

bpy.ops.mesh.primitive_cube_add(size=1.0, location=(16.8, 0.0, 2.5))
wall_e = bpy.context.active_object
wall_e.name = "Occluder_GasStation_Wall_East"
wall_e.scale = (0.4, 10.0, 5.0)

bpy.ops.mesh.primitive_cube_add(size=1.0, location=(10.0, 4.8, 2.5))
wall_n = bpy.context.active_object
wall_n.name = "Occluder_GasStation_Wall_North"
wall_n.scale = (14.0, 0.4, 5.0)

bpy.ops.mesh.primitive_cube_add(size=1.0, location=(5.5, -4.8, 2.5))
wall_sa = bpy.context.active_object
wall_sa.name = "Occluder_GasStation_Wall_South_A"
wall_sa.scale = (5.0, 0.4, 5.0)

bpy.ops.mesh.primitive_cube_add(size=1.0, location=(14.5, -4.8, 2.5))
wall_sb = bpy.context.active_object
wall_sb.name = "Occluder_GasStation_Wall_South_B"
wall_sb.scale = (5.0, 0.4, 5.0)

# 4. Gas station Roof (tagged for interior cutaway occlusion: glTF Y=5.2)
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(10.0, 0.0, 5.2))
roof = bpy.context.active_object
roof.name = "Roof_GasStation"
roof.scale = (15.0, 11.0, 0.4)

# 5. Gas station Interior Trigger volume (glTF Y=2.5)
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(10.0, 0.0, 2.5))
trigger = bpy.context.active_object
trigger.name = "Trigger_Interior_GasStation"
trigger.scale = (13.5, 9.5, 4.8)

# 6. Low-poly Colliders
# Wall West (glTF X=-30, glTF Y=2.0, glTF Z=0; length 60m along glTF Z)
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(-30.0, 0.0, 2.0))
col_w = bpy.context.active_object
col_w.name = "Collider_Wall_West"
col_w.scale = (1.0, 60.0, 4.0)

# Wall East (glTF X=30, glTF Y=2.0, glTF Z=0; length 60m along glTF Z)
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(30.0, 0.0, 2.0))
col_e = bpy.context.active_object
col_e.name = "Collider_Wall_East"
col_e.scale = (1.0, 60.0, 4.0)

# Concrete Barrier 01 (glTF X=-5, glTF Y=0.6, glTF Z=-10 -> Y_blend=10, Z_blend=0.6)
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(-5.0, 10.0, 0.6))
bar1 = bpy.context.active_object
bar1.name = "Collider_Barrier_01"
bar1.scale = (4.0, 1.0, 1.2)

# Concrete Barrier 02 (glTF X=5, glTF Y=0.6, glTF Z=12 -> Y_blend=-12, Z_blend=0.6)
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(5.0, -12.0, 0.6))
bar2 = bpy.context.active_object
bar2.name = "Collider_Barrier_02"
bar2.scale = (4.0, 1.0, 1.2)

# 7. Gameplay Empties / Markers (Elevation Z_blend=0.0 -> glTF Y=0.0)
# Player Spawn (South-West: glTF X=-20, glTF Z=20 -> Y_blend=-20)
bpy.ops.object.empty_add(type='PLAIN_AXES', location=(-20.0, -20.0, 0.0))
spawn_p = bpy.context.active_object
spawn_p.name = "Spawn_Player"

# Enemy Spawners
# Spawn_Enemy_Walker (North-East: glTF X=20, glTF Z=-20 -> Y_blend=20)
bpy.ops.object.empty_add(type='PLAIN_AXES', location=(20.0, 20.0, 0.0))
sp_e1 = bpy.context.active_object
sp_e1.name = "Spawn_Enemy_Walker"

# Spawn_Enemy_Runner (North-West: glTF X=-20, glTF Z=-20 -> Y_blend=20)
bpy.ops.object.empty_add(type='PLAIN_AXES', location=(-20.0, 20.0, 0.0))
sp_e2 = bpy.context.active_object
sp_e2.name = "Spawn_Enemy_Runner"

# Extraction Zone (North highway end: glTF X=0, glTF Z=-35 -> Y_blend=35)
bpy.ops.object.empty_add(type='PLAIN_AXES', location=(0.0, 35.0, 0.0))
ext_z = bpy.context.active_object
ext_z.name = "Extract_Zone"

# Apply all scale transforms to meshes
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        bpy.context.view_layer.objects.active = obj
        obj.select_set(True)
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
        obj.select_set(False)

# Export to GLB
output_path = "E:/Codes/AGameProject/game/assets/maps/district_01.glb"
bpy.ops.export_scene.gltf(filepath=output_path, export_format='GLB')
print("Successfully generated and exported district_01.glb!")

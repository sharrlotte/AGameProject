import bpy

# Reset scene to empty
bpy.ops.wm.read_factory_settings(use_empty=True)

# 1. Ground plane (Visual)
bpy.ops.mesh.primitive_plane_add(size=120.0, location=(0, 0, 0))
ground = bpy.context.active_object
ground.name = "Visual_Ground"

# 2. Road surface (Visual)
bpy.ops.mesh.primitive_plane_add(size=20.0, location=(0, 0.02, 0))
road = bpy.context.active_object
road.name = "Visual_Road_Main"
road.scale = (1.0, 5.0, 1.0)

# 3. Gas station building visual exterior walls
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(10.0, 2.5, 0.0))
bldg = bpy.context.active_object
bldg.name = "Visual_GasStation_Building"
bldg.scale = (14.0, 5.0, 10.0)

# 4. Gas station Roof (tagged for interior cutaway occlusion)
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(10.0, 5.2, 0.0))
roof = bpy.context.active_object
roof.name = "Roof_GasStation"
roof.scale = (15.0, 0.4, 11.0)

# 5. Gas station Interior Trigger volume
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(10.0, 2.5, 0.0))
trigger = bpy.context.active_object
trigger.name = "Trigger_Interior_GasStation"
trigger.scale = (13.5, 4.8, 9.5)

# 6. Low-poly Colliders
# Wall West
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(-30.0, 2.0, 0.0))
col_w = bpy.context.active_object
col_w.name = "Collider_Wall_West"
col_w.scale = (1.0, 4.0, 60.0)

# Wall East
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(30.0, 2.0, 0.0))
col_e = bpy.context.active_object
col_e.name = "Collider_Wall_East"
col_e.scale = (1.0, 4.0, 60.0)

# Concrete Barrier 01
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(-5.0, 0.6, -10.0))
bar1 = bpy.context.active_object
bar1.name = "Collider_Barrier_01"
bar1.scale = (4.0, 1.2, 1.0)

# Concrete Barrier 02
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(5.0, 0.6, 12.0))
bar2 = bpy.context.active_object
bar2.name = "Collider_Barrier_02"
bar2.scale = (4.0, 1.2, 1.0)

# 7. Gameplay Empties / Markers
# Player Spawn
bpy.ops.object.empty_add(type='PLAIN_AXES', location=(-20.0, 0.0, -20.0))
spawn_p = bpy.context.active_object
spawn_p.name = "Spawn_Player"

# Enemy Spawners
bpy.ops.object.empty_add(type='PLAIN_AXES', location=(20.0, 0.0, 20.0))
sp_e1 = bpy.context.active_object
sp_e1.name = "Spawn_Enemy_Walker"

bpy.ops.object.empty_add(type='PLAIN_AXES', location=(-20.0, 0.0, 20.0))
sp_e2 = bpy.context.active_object
sp_e2.name = "Spawn_Enemy_Runner"

# Extraction Zone
bpy.ops.object.empty_add(type='PLAIN_AXES', location=(0.0, 0.0, 35.0))
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

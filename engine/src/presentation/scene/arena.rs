use bevy::prelude::*;
use crate::config::ConfigHotReloadedMessage;
use crate::world::{ArenaConfig, ArenaGround, ArenaObstacle};

use super::material::SharedVisualMaterials;

#[derive(Component)]
pub struct SunLightTag;

pub fn setup_lighting(
    mut commands: Commands,
    cfg: Res<ArenaConfig>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    ambient.color = Color::srgb(0.85, 0.90, 1.0);
    ambient.brightness = cfg.ambient_brightness;

    commands.spawn((
        Name::new("SunLight"),
        SunLightTag,
        DirectionalLight {
            illuminance: cfg.sun_illuminance,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(18.0, 32.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn sync_lighting_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<ArenaConfig>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut suns: Query<&mut DirectionalLight, With<SunLightTag>>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }
    ambient.brightness = cfg.ambient_brightness;
    for mut sun in &mut suns {
        sun.illuminance = cfg.sun_illuminance;
    }
}

pub fn attach_arena_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    cfg: Res<ArenaConfig>,
    materials: Res<SharedVisualMaterials>,
    grounds: Query<Entity, Added<ArenaGround>>,
    obstacles: Query<(Entity, &ArenaObstacle), Added<ArenaObstacle>>,
) {
    for entity in &grounds {
        let size = cfg.half_size * 2.0;
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(size, 1.0, size))),
            MeshMaterial3d(materials.ground.clone()),
        ));
    }

    for (entity, obstacle) in &obstacles {
        let ext = obstacle.half_extents * 2.0;
        let mat = if obstacle.is_wall {
            materials.wall.clone()
        } else {
            materials.cover.clone()
        };
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(ext.x, ext.y, ext.z))),
            MeshMaterial3d(mat),
        ));
    }
}








use bevy::prelude::*;

use crate::sim::arena::{ARENA_HALF_SIZE, ArenaGround, ArenaObstacle};
use crate::sim::combat::Projectile;
use crate::sim::enemies::{Enemy, EnemyKind};
use crate::sim::player::{PLAYER_HALF_HEIGHT, PLAYER_RADIUS, Survivor};
use crate::sim::DamageMessage;

const HIT_FLASH_DURATION: f32 = 0.09;

#[derive(Resource)]
pub struct SharedVisualMaterials {
    pub ground: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub cover: Handle<StandardMaterial>,
    pub player: Handle<StandardMaterial>,
    pub player_barrel: Handle<StandardMaterial>,
    pub walker: Handle<StandardMaterial>,
    pub runner: Handle<StandardMaterial>,
    pub projectile: Handle<StandardMaterial>,
    pub hit_flash: Handle<StandardMaterial>,
}

impl FromWorld for SharedVisualMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self {
            ground: materials.add(StandardMaterial {
                base_color: Color::srgb(0.14, 0.16, 0.19),
                perceptual_roughness: 0.9,
                ..default()
            }),
            wall: materials.add(StandardMaterial {
                base_color: Color::srgb(0.26, 0.29, 0.34),
                perceptual_roughness: 0.85,
                ..default()
            }),
            cover: materials.add(StandardMaterial {
                base_color: Color::srgb(0.55, 0.45, 0.30),
                perceptual_roughness: 0.75,
                ..default()
            }),
            player: materials.add(StandardMaterial {
                base_color: Color::srgb(0.20, 0.62, 0.95),
                perceptual_roughness: 0.45,
                ..default()
            }),
            player_barrel: materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.14, 0.18),
                metallic: 0.6,
                perceptual_roughness: 0.35,
                ..default()
            }),
            walker: materials.add(StandardMaterial {
                base_color: Color::srgb(0.36, 0.62, 0.30),
                perceptual_roughness: 0.7,
                ..default()
            }),
            runner: materials.add(StandardMaterial {
                base_color: Color::srgb(0.90, 0.30, 0.22),
                perceptual_roughness: 0.6,
                ..default()
            }),
            projectile: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.88, 0.35),
                emissive: LinearRgba::rgb(4.0, 3.0, 0.6),
                ..default()
            }),
            hit_flash: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::rgb(3.5, 3.5, 3.5),
                ..default()
            }),
        }
    }
}

#[derive(Component)]
pub struct HitFlash {
    pub timer: f32,
    pub base_material: Handle<StandardMaterial>,
}

/// Spawns directional sunlight and sets global ambient lighting.
pub fn setup_lighting(mut commands: Commands, mut ambient: ResMut<GlobalAmbientLight>) {
    ambient.color = Color::srgb(0.85, 0.90, 1.0);
    ambient.brightness = 280.0;

    commands.spawn((
        Name::new("SunLight"),
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(18.0, 32.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Attaches 3D meshes and materials to newly spawned static arena entities.
pub fn attach_arena_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SharedVisualMaterials>,
    grounds: Query<Entity, Added<ArenaGround>>,
    obstacles: Query<(Entity, &ArenaObstacle), Added<ArenaObstacle>>,
) {
    for entity in &grounds {
        let size = ARENA_HALF_SIZE * 2.0;
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

/// Attaches 3D meshes and facing barrel indicators to newly spawned survivors, enemies, and projectiles.
pub fn attach_actor_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SharedVisualMaterials>,
    survivors: Query<Entity, Added<Survivor>>,
    enemies: Query<(Entity, &Enemy), Added<Enemy>>,
    projectiles: Query<(Entity, &Projectile), Added<Projectile>>,
) {
    for entity in &survivors {
        let body_mesh = meshes.add(Capsule3d::new(PLAYER_RADIUS, PLAYER_HALF_HEIGHT * 2.0));
        let barrel_mesh = meshes.add(Cuboid::new(0.14, 0.14, 0.72));
        commands
            .entity(entity)
            .insert((
                Mesh3d(body_mesh),
                MeshMaterial3d(materials.player.clone()),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(barrel_mesh),
                    MeshMaterial3d(materials.player_barrel.clone()),
                    Transform::from_xyz(0.16, 0.12, -0.45),
                ));
            });
    }

    for (entity, enemy) in &enemies {
        let body_mesh = meshes.add(Capsule3d::new(enemy.radius, enemy.half_height * 2.0));
        let base_mat = match enemy.kind {
            EnemyKind::Walker => materials.walker.clone(),
            EnemyKind::Runner => materials.runner.clone(),
        };
        commands.entity(entity).insert((
            Mesh3d(body_mesh),
            MeshMaterial3d(base_mat.clone()),
            HitFlash {
                timer: 0.0,
                base_material: base_mat,
            },
        ));
    }

    for (entity, proj) in &projectiles {
        let sphere_mesh = meshes.add(Sphere::new(proj.radius));
        commands.entity(entity).insert((
            Mesh3d(sphere_mesh),
            MeshMaterial3d(materials.projectile.clone()),
        ));
    }
}

/// Triggers a brief white hit-flash on struck zombies and restores their base material.
pub fn update_hit_flash(
    time: Res<Time>,
    materials: Res<SharedVisualMaterials>,
    mut damage_reader: MessageReader<DamageMessage>,
    mut flashes: Query<(&mut HitFlash, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    for msg in damage_reader.read() {
        if let Ok((mut flash, mut mat)) = flashes.get_mut(msg.target) {
            flash.timer = HIT_FLASH_DURATION;
            mat.0 = materials.hit_flash.clone();
        }
    }

    let dt = time.delta_secs();
    for (mut flash, mut mat) in &mut flashes {
        if flash.timer > 0.0 {
            flash.timer = (flash.timer - dt).max(0.0);
            if flash.timer <= 0.0 {
                mat.0 = flash.base_material.clone();
            }
        }
    }
}

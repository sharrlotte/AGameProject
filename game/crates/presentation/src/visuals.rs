use bevy::prelude::*;
use combat::Projectile;
use data::{
    ConfigHotReloadedMessage, DamageMessage, EnemyKind, GameState, SimPostPhysicsSet, SimUpdateSet,
};
use enemies::{EnemiesConfig, Enemy, FacingDirection};
use extraction::{ExtractionZone, ScrapPickup};
use world::{ArenaConfig, ArenaGround, ArenaObstacle, Survivor};

use crate::anim::PlayerVisualRoot;

#[derive(Component)]
pub struct VisualBody;

#[derive(Component)]
pub struct SunLightTag;

#[derive(Component)]
pub struct ExtractionPadVisual;

#[derive(Component)]
pub struct HitFlash {
    pub timer: f32,
    pub base_material: Handle<StandardMaterial>,
    pub visual_entity: Entity,
}

#[derive(Resource)]
pub struct SharedVisualMaterials {
    pub ground: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub cover: Handle<StandardMaterial>,
    pub walker: Handle<StandardMaterial>,
    pub runner: Handle<StandardMaterial>,
    pub projectile: Handle<StandardMaterial>,
    pub hit_flash: Handle<StandardMaterial>,
    pub scrap: Handle<StandardMaterial>,
    pub extraction_idle: Handle<StandardMaterial>,
    pub extraction_active: Handle<StandardMaterial>,
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
            scrap: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.74, 0.18),
                emissive: LinearRgba::rgb(1.8, 1.1, 0.15),
                metallic: 0.5,
                perceptual_roughness: 0.3,
                ..default()
            }),
            extraction_idle: materials.add(StandardMaterial {
                base_color: Color::srgba(0.15, 0.82, 0.75, 0.65),
                emissive: LinearRgba::rgb(0.2, 1.2, 1.0),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            extraction_active: materials.add(StandardMaterial {
                base_color: Color::srgba(0.25, 1.0, 0.55, 0.85),
                emissive: LinearRgba::rgb(0.5, 2.8, 1.2),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        }
    }
}

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SharedVisualMaterials>()
            .add_systems(OnExit(GameState::Loading), setup_lighting)
            .add_systems(
                Update,
                (
                    sync_lighting_hot_reload,
                    attach_arena_visuals,
                    attach_actor_visuals,
                    attach_extraction_visuals,
                    animate_scrap_pickups,
                    update_extraction_pad_visual,
                )
                    .chain()
                    .after(SimUpdateSet)
                    .run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                PostUpdate,
                (sync_visual_facing, update_hit_flash)
                    .after(SimPostPhysicsSet)
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_lighting(
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

fn sync_lighting_hot_reload(
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

fn attach_arena_visuals(
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

fn attach_actor_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SharedVisualMaterials>,
    survivors: Query<Entity, Added<Survivor>>,
    enemies: Query<(Entity, &Enemy), Added<Enemy>>,
    projectiles: Query<(Entity, &Projectile), Added<Projectile>>,
    pickups: Query<(Entity, &ScrapPickup), Added<ScrapPickup>>,
) {
    for entity in &survivors {
        commands
            .entity(entity)
            .insert(Visibility::default())
            .with_children(|parent| {
                parent.spawn((
                    Name::new("PlayerVisualRoot"),
                    PlayerVisualRoot,
                    Transform::default(),
                    Visibility::default(),
                ));
            });
    }

    for (entity, enemy) in &enemies {
        let body_mesh = meshes.add(Capsule3d::new(enemy.radius, enemy.half_height * 2.0));
        let base_mat = match enemy.kind {
            EnemyKind::Walker => materials.walker.clone(),
            EnemyKind::Runner => materials.runner.clone(),
        };
        let visual_entity = commands
            .spawn((
                VisualBody,
                Mesh3d(body_mesh),
                MeshMaterial3d(base_mat.clone()),
                Transform::default(),
            ))
            .id();
        commands
            .entity(entity)
            .insert((
                Visibility::default(),
                HitFlash {
                    timer: 0.0,
                    base_material: base_mat,
                    visual_entity,
                },
            ))
            .add_child(visual_entity);
    }

    for (entity, proj) in &projectiles {
        let sphere_mesh = meshes.add(Sphere::new(proj.radius));
        commands.entity(entity).insert((
            Mesh3d(sphere_mesh),
            MeshMaterial3d(materials.projectile.clone()),
        ));
    }

    for (entity, pickup) in &pickups {
        let side = pickup.radius * 1.6;
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(side, side, side))),
            MeshMaterial3d(materials.scrap.clone()),
        ));
    }
}

fn attach_extraction_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SharedVisualMaterials>,
    zones: Query<(Entity, &ExtractionZone), Added<ExtractionZone>>,
) {
    for (entity, zone) in &zones {
        commands.entity(entity).insert((
            ExtractionPadVisual,
            Mesh3d(meshes.add(Cylinder::new(zone.radius, 0.06))),
            MeshMaterial3d(materials.extraction_idle.clone()),
        ));
    }
}

fn update_extraction_pad_visual(
    materials: Res<SharedVisualMaterials>,
    mut pads: Query<
        (&ExtractionZone, &mut MeshMaterial3d<StandardMaterial>),
        With<ExtractionPadVisual>,
    >,
) {
    for (zone, mut mat) in &mut pads {
        mat.0 = if zone.active {
            materials.extraction_active.clone()
        } else {
            materials.extraction_idle.clone()
        };
    }
}

fn animate_scrap_pickups(time: Res<Time>, mut pickups: Query<&mut Transform, With<ScrapPickup>>) {
    let dt = time.delta_secs();
    for mut tf in &mut pickups {
        tf.rotate_y(2.4 * dt);
    }
}

fn sync_visual_facing(
    enemies: Query<(&FacingDirection, &Children), With<Enemy>>,
    mut visual_bodies: Query<&mut Transform, With<VisualBody>>,
) {
    for (facing, children) in &enemies {
        apply_facing_to_children(facing.0, children, &mut visual_bodies);
    }
}

fn apply_facing_to_children(
    dir: Vec3,
    children: &Children,
    visual_bodies: &mut Query<&mut Transform, With<VisualBody>>,
) {
    if dir.length_squared() <= 1e-4 {
        return;
    }
    let rot = Quat::from_rotation_arc(Vec3::NEG_Z, dir.normalize());
    for child in children.iter() {
        if let Ok(mut tf) = visual_bodies.get_mut(child) {
            tf.rotation = rot;
        }
    }
}

fn update_hit_flash(
    time: Res<Time>,
    enemies_cfg: Res<EnemiesConfig>,
    materials: Res<SharedVisualMaterials>,
    mut damage_reader: MessageReader<DamageMessage>,
    mut flashes: Query<&mut HitFlash>,
    mut mesh_materials: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    for msg in damage_reader.read() {
        if let Ok(mut flash) = flashes.get_mut(msg.target) {
            flash.timer = enemies_cfg.hit_flash_duration;
            if let Ok(mut mat) = mesh_materials.get_mut(flash.visual_entity) {
                mat.0 = materials.hit_flash.clone();
            }
        }
    }

    let dt = time.delta_secs();
    for mut flash in &mut flashes {
        if flash.timer > 0.0 {
            flash.timer = (flash.timer - dt).max(0.0);
            if flash.timer <= 0.0
                && let Ok(mut mat) = mesh_materials.get_mut(flash.visual_entity)
            {
                mat.0 = flash.base_material.clone();
            }
        }
    }
}

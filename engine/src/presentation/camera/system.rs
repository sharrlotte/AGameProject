use std::collections::HashSet;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::common::{GROUP_WORLD, Obstacle};
use crate::config::ConfigHotReloadedMessage;
use crate::world::{ActionIntent, LocalPlayer};

use crate::config::CameraConfig;
use crate::message::messages::CameraZoomMessage;
use crate::presentation::scene::SharedVisualMaterials;
use super::state::{CameraZoom, MainCamera, OccludedWall};

/// Spawns the primary 3D camera using the configured offset and default zoom.
pub fn spawn_camera(mut commands: Commands, cfg: Res<CameraConfig>) {
    let zoom = CameraZoom::from_config(&cfg);
    let offset = cfg.offset * zoom.current;
    commands.spawn((
        Name::new("MainCamera"),
        MainCamera,
        zoom,
        Camera3d::default(),
        Transform::from_translation(offset).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Applies incoming [`CameraZoomMessage`] steps to the main camera's target zoom.
pub fn apply_camera_zoom(
    mut zoom_reader: MessageReader<CameraZoomMessage>,
    cfg: Res<CameraConfig>,
    mut zoom: Single<&mut CameraZoom, With<MainCamera>>,
) {
    for msg in zoom_reader.read() {
        zoom.adjust_target(msg.steps, &cfg);
    }
}

/// Re-orients the camera and clamps zoom bounds when [`CameraConfig`] hot-reloads.
pub fn sync_camera_on_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<CameraConfig>,
    camera: Single<(&mut Transform, &mut CameraZoom), With<MainCamera>>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }
    let (mut tf, mut zoom) = camera.into_inner();
    zoom.clamp_to_config(&cfg);
    let scaled_offset = cfg.offset * zoom.current;
    let focus = tf.translation - scaled_offset;
    *tf = Transform::from_translation(focus + scaled_offset).looking_at(focus, Vec3::Y);
}

/// Smoothly follows the local player with aim-lead and exponential zoom damping.
#[allow(clippy::type_complexity)]
pub fn follow_local_player(
    time: Res<Time>,
    cfg: Res<CameraConfig>,
    player: Single<(&Transform, &ActionIntent), With<LocalPlayer>>,
    camera: Single<(&mut Transform, &mut CameraZoom), (With<MainCamera>, Without<LocalPlayer>)>,
) {
    let (player_tf, intent) = *player;
    let (mut cam_tf, mut zoom) = camera.into_inner();

    let mut aim_lead = Vec3::ZERO;
    if let Some(aim_world) = intent.aim_world {
        let delta = Vec3::new(
            aim_world.x - player_tf.translation.x,
            0.0,
            aim_world.z - player_tf.translation.z,
        );
        let scaled = delta * cfg.aim_lead_factor;
        let len = scaled.length();
        aim_lead = if len > cfg.max_aim_lead {
            scaled * (cfg.max_aim_lead / len)
        } else {
            scaled
        };
    }

    let dt = time.delta_secs();
    let zoom_alpha = 1.0 - (-cfg.zoom_sharpness * dt).exp();
    zoom.current += (zoom.target - zoom.current) * zoom_alpha;

    let focus = Vec3::new(player_tf.translation.x, 0.0, player_tf.translation.z) + aim_lead;
    let target_pos = focus + cfg.offset * zoom.current;
    let follow_alpha = 1.0 - (-cfg.follow_sharpness * dt).exp();

    // Keep camera orientation fixed; only translate so follow lag never tilts/jiggles the view.
    cam_tf.translation = cam_tf.translation.lerp(target_pos, follow_alpha);
}

/// Casts a ray from the camera toward the survivor to detect occluding world obstacles,
/// swapping their material to a transparent blend and restoring original materials when clear.
#[allow(clippy::type_complexity)]
pub fn update_camera_wall_occlusion(
    mut commands: Commands,
    rapier_context: ReadRapierContext,
    materials: Option<Res<SharedVisualMaterials>>,
    camera: Single<&GlobalTransform, With<MainCamera>>,
    player: Query<&GlobalTransform, With<LocalPlayer>>,
    obstacles: Query<(), With<Obstacle>>,
    mut occluded_walls: Query<(Entity, &OccludedWall, &mut MeshMaterial3d<StandardMaterial>)>,
    unoccluded_materials: Query<&MeshMaterial3d<StandardMaterial>, Without<OccludedWall>>,
    children_query: Query<&Children>,
) {
    let Ok(rapier) = rapier_context.single() else {
        return;
    };
    let Some(materials) = materials else {
        return;
    };
    let Ok(player_tf) = player.single() else {
        return;
    };

    let cam_pos = camera.translation();
    let player_pos = player_tf.translation();
    let to_player = player_pos - cam_pos;
    let distance = to_player.length();
    if distance < 0.5 {
        return;
    }
    let ray_dir = to_player / distance;

    let mut filter = QueryFilter::default().groups(CollisionGroups::new(GROUP_WORLD, GROUP_WORLD));
    let mut currently_occluded = HashSet::new();
    let remaining_dist = distance - 0.2;

    for _ in 0..4 {
        if let Some((hit_entity, _toi)) = rapier.cast_ray(cam_pos, ray_dir, remaining_dist, true, filter) {
            filter = filter.exclude_collider(hit_entity);
            if obstacles.contains(hit_entity) {
                // Find visual entity with MeshMaterial3d (either the obstacle entity itself or a child)
                let target_visual = if unoccluded_materials.contains(hit_entity) || occluded_walls.contains(hit_entity) {
                    Some(hit_entity)
                } else if let Ok(children) = children_query.get(hit_entity) {
                    children.iter().find(|c| unoccluded_materials.contains(*c) || occluded_walls.contains(*c))
                } else {
                    None
                };

                if let Some(visual_entity) = target_visual {
                    currently_occluded.insert(visual_entity);
                    if let Ok(mat) = unoccluded_materials.get(visual_entity) {
                        commands.entity(visual_entity).insert((
                            OccludedWall {
                                original_material: mat.0.clone(),
                            },
                            MeshMaterial3d(materials.wall_occluded.clone()),
                        ));
                    }
                }
            }
        } else {
            break;
        }
    }

    // Restore walls that are no longer occluding
    for (entity, occluded, mut mat) in &mut occluded_walls {
        if !currently_occluded.contains(&entity) {
            mat.0 = occluded.original_material.clone();
            commands.entity(entity).remove::<OccludedWall>();
        }
    }
}

#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn zoom_adjust_respects_direction_and_clamps_bounds() {
        let cfg = CameraConfig::default();
        let mut zoom = CameraZoom::from_config(&cfg);

        // Positive steps zoom in (decrease multiplier).
        zoom.adjust_target(1.0, &cfg);
        assert!((zoom.target - (cfg.zoom_default - cfg.zoom_step)).abs() < 1e-5);

        // Massive zoom-in clamps to zoom_min.
        zoom.adjust_target(100.0, &cfg);
        assert!((zoom.target - cfg.zoom_min).abs() < 1e-5);

        // Massive zoom-out clamps to zoom_max.
        zoom.adjust_target(-100.0, &cfg);
        assert!((zoom.target - cfg.zoom_max).abs() < 1e-5);
    }

    #[test]
    fn apply_camera_zoom_message_updates_target_zoom() {
        let mut app = App::new();
        app.add_message::<CameraZoomMessage>()
            .init_resource::<CameraConfig>()
            .add_systems(Update, apply_camera_zoom);

        let cfg = app.world().resource::<CameraConfig>().clone();
        let cam = app
            .world_mut()
            .spawn((MainCamera, CameraZoom::from_config(&cfg)))
            .id();

        app.world_mut().write_message(CameraZoomMessage { steps: 2.0 });
        app.update();

        let zoom = app.world().get::<CameraZoom>(cam).expect("camera zoom");
        assert!((zoom.target - (cfg.zoom_default - 2.0 * cfg.zoom_step)).abs() < 1e-5);
    }

    #[test]
    fn camera_wall_occlusion_swaps_material_and_restores() {
        use std::time::Duration;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin))
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .insert_resource(TimestepMode::Variable {
                max_dt: 1.0 / 60.0,
                time_scale: 1.0,
                substeps: 1,
            })
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ))
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Assets<Mesh>>();

        let materials = SharedVisualMaterials::from_world(app.world_mut());
        let original_mat = materials.wall.clone();
        let occluded_mat = materials.wall_occluded.clone();
        app.insert_resource(materials)
            .add_systems(Update, update_camera_wall_occlusion);

        // Spawn player at (0, 0, 0)
        app.world_mut().spawn((
            LocalPlayer,
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Spawn camera at (0, 10, 10)
        app.world_mut().spawn((
            MainCamera,
            Camera3d::default(),
            Transform::from_xyz(0.0, 10.0, 10.0),
            GlobalTransform::from_xyz(0.0, 10.0, 10.0),
        ));

        // Spawn obstacle wall between camera and player at (0, 5, 5)
        let wall = app.world_mut().spawn((
            Obstacle,
            RigidBody::Fixed,
            Collider::cuboid(2.0, 2.0, 2.0),
            CollisionGroups::new(GROUP_WORLD, GROUP_WORLD),
            Transform::from_xyz(0.0, 5.0, 5.0),
            GlobalTransform::from_xyz(0.0, 5.0, 5.0),
            MeshMaterial3d(original_mat.clone()),
        )).id();

        app.update();
        app.update();
        app.update();

        // Wall should now be occluded
        assert!(app.world().get::<OccludedWall>(wall).is_some());
        assert_eq!(
            app.world().get::<MeshMaterial3d<StandardMaterial>>(wall).unwrap().0,
            occluded_mat
        );

        // Move wall out of the line of sight to (20, 5, 5)
        app.world_mut().get_mut::<Transform>(wall).unwrap().translation = Vec3::new(20.0, 5.0, 5.0);
        *app.world_mut().get_mut::<GlobalTransform>(wall).unwrap() = GlobalTransform::from_xyz(20.0, 5.0, 5.0);

        app.update();
        app.update();
        app.update();

        // Wall should be restored
        assert!(app.world().get::<OccludedWall>(wall).is_none());
        assert_eq!(
            app.world().get::<MeshMaterial3d<StandardMaterial>>(wall).unwrap().0,
            original_mat
        );
    }
}

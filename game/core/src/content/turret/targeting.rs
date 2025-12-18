use crate::content::{
    structures::{block::Block, ship::Enemy},
    team::Team,
    turret::{AIFireController, PlayerFireController, TargetingStrategy, TurretController},
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn update_player_fire_controller(
    mut query: Query<(&mut TurretController, &GlobalTransform), With<PlayerFireController>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            for (mut controller, _) in query.iter_mut() {
                controller.target_pos = Some(world_position);
                controller.active = mouse_input.pressed(MouseButton::Left);
            }
        }
    }
}

pub fn update_turret_target(
    mut collision_events: MessageReader<CollisionEvent>,
    turrets: Query<(Entity, &GlobalTransform, &TurretController, &Team)>,
    blocks: Query<(Entity, &GlobalTransform, &Block)>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(e1, e2, _flags) => {
                // Check if e1 or e2 is a DetectionRange sensor
                let (turret, block) = match (
                    turrets.contains(*e1),
                    blocks.contains(*e1),
                    turrets.contains(*e2),
                    blocks.contains(*e2),
                ) {
                    (true, _, _, true) => (turrets.get_mut(*e1), blocks.get_mut(*e2)), // e1 = sensor, e2 = enemy
                    (_, true, true, _) => (turrets.get_mut(*e2), blocks.get_mut(*e1)), // e2 = sensor, e1 = enemy
                    _ => continue,
                };

                // Add enemy to potential targets
                if let Ok(((enemy, _))) = block
                    && let Ok((_, _, mut turretc)) = turret
                {
                    turretc.potential_targets.push(enemy);
                }
            }
            CollisionEvent::Stopped(e1, e2, _flags) => {
                let (turret, block) = match (
                    turrets.contains(*e1),
                    blocks.contains(*e1),
                    turrets.contains(*e2),
                    blocks.contains(*e2),
                ) {
                    (true, _, _, true) => (turrets.get_mut(*e1), blocks.get_mut(*e2)), // e1 = sensor, e2 = enemy
                    (_, true, true, _) => (turrets.get_mut(*e2), blocks.get_mut(*e1)), // e2 = sensor, e1 = enemy
                    _ => continue,
                };

                if let Ok(((mut enemy, _))) = block
                    && let Ok((_, _, mut turretc)) = turret
                {
                    turretc.potential_targets.remove(&enemy);
                }
            }
        }
    }

    // 2. Select best target from potential_targets
    for (entity, transform, mut controller, team) in turrets.iter_mut() {
        let turret_pos = ransform.translation().truncate();
        
        let mut best_target: Option<Entity> = None;
        let mut closest_dist_sq = f32::MAX; // Use MAX to ensure we check all valid targets within logical range

        for &entity in controller.potential_targets.iter() {
            if let Ok((_, target_transform, _)) = blocks.get(entity) {
                let target_pos_actual = target_transform.translation().truncate();
                let dist_sq = turret_pos.distance_squared(target_pos_actual);

                if dist_sq > controller.range * controller.range {
                    continue;
                }

                match controller.strategy {
                    TargetingStrategy::Closest => {
                        if dist_sq < closest_dist_sq {
                            closest_dist_sq = dist_sq;
                            best_target = Some(entity);
                        }
                    }
                    TargetingStrategy::MostHP => {
                        // Placeholder logic
                        if dist_sq < closest_dist_sq {
                            closest_dist_sq = dist_sq;
                            best_target = Some(entity);
                        }
                    }
                    TargetingStrategy::EnemyBullet => {
                        // Placeholder logic
                        if dist_sq < closest_dist_sq {
                            closest_dist_sq = dist_sq;
                            best_target = Some(entity);
                        }
                    }
                }
            }
        }

        if let Some(entity) = best_target {
            if let Ok(target_pos) = blocks
                .get(entity)
                .map(|op| op.1.translation().truncate())
            {
                controller.target_pos = Some(target_pos);
                controller.active = true;
            } else {
                controller.target_pos = None;
                controller.active = false;
            }
        }
    }
}

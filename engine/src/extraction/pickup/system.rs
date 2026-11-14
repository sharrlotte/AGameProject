use bevy::prelude::*;
use crate::common::{RunEntity, GameScene};
use crate::message::messages::{EnemyKilledMessage, ScrapCollectedMessage};
use crate::world::Survivor;

use crate::config::ExtractionConfig;

use super::state::ScrapPickup;

pub fn spawn_scrap_on_enemy_killed(
    mut commands: Commands,
    cfg: Res<ExtractionConfig>,
    mut killed_reader: MessageReader<EnemyKilledMessage>,
) {
    for msg in killed_reader.read() {
        let amount = msg.scrap;
        if amount == 0 {
            continue;
        }
        commands.spawn((
            Name::new("ScrapPickup"),
            RunEntity,
            ScrapPickup {
                amount,
                radius: cfg.scrap_radius,
                lifetime: cfg.scrap_lifetime_secs,
            },
            Transform::from_xyz(msg.position.x, cfg.scrap_radius + 0.05, msg.position.z),
            DespawnOnExit(GameScene::Arena),
        ));
    }
}

pub fn collect_scrap_pickups(
    mut commands: Commands,
    time: Res<Time>,
    cfg: Res<ExtractionConfig>,
    mut scrap_writer: MessageWriter<ScrapCollectedMessage>,
    survivor: Option<Single<(Entity, &Transform), With<Survivor>>>,
    mut pickups: Query<(Entity, &mut ScrapPickup, &mut Transform), Without<Survivor>>,
) {
    let dt = time.delta_secs();
    let survivor_data = survivor.as_ref().map(|s| {
        let (e, t) = **s;
        (e, t.translation)
    });

    for (pickup_entity, mut pickup, mut transform) in &mut pickups {
        pickup.lifetime -= dt;
        if pickup.lifetime <= 0.0 {
            commands.entity(pickup_entity).despawn();
            continue;
        }

        let Some((collector, target)) = survivor_data else {
            continue;
        };

        let pos = transform.translation;
        let delta = Vec3::new(target.x - pos.x, 0.0, target.z - pos.z);
        let dist = delta.length();
        if dist <= cfg.scrap_pickup_radius {
            scrap_writer.write(ScrapCollectedMessage {
                collector,
                amount: pickup.amount,
            });
            commands.entity(pickup_entity).despawn();
        } else if dist <= cfg.scrap_magnet_radius && dist > 1e-4 {
            let step = (cfg.scrap_magnet_speed * dt).min(dist);
            transform.translation += (delta / dist) * step;
        }
    }
}


use bevy::prelude::*;
use crate::config::ConfigHotReloadedMessage;
use crate::common::{RunEntity, GameScene};
use crate::message::messages::ExtractionCompletedMessage;
use crate::message::resources::RunStats;
use crate::world::Survivor;

use crate::config::ExtractionConfig;

use super::state::ExtractionZone;

pub fn spawn_extraction_zone(
    mut commands: Commands,
    cfg: Res<ExtractionConfig>,
    existing: Query<(), With<ExtractionZone>>,
) {
    if !existing.is_empty() {
        return;
    }
    commands.spawn((
        Name::new("ExtractionZone"),
        RunEntity,
        ExtractionZone {
            radius: cfg.pad_radius,
            hold_duration_secs: cfg.hold_duration_secs,
            progress_secs: 0.0,
            active: false,
            completed: false,
        },
        Transform::from_translation(cfg.pad_center),
        DespawnOnExit(GameScene::Arena),
    ));
}

pub fn sync_extraction_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<ExtractionConfig>,
    mut zone: Single<(&mut ExtractionZone, &mut Transform)>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }

    zone.0.radius = cfg.pad_radius;
    zone.0.hold_duration_secs = cfg.hold_duration_secs;
    zone.1.translation = cfg.pad_center;
}

pub fn update_extraction_zone(
    time: Res<Time>,
    mut stats: ResMut<RunStats>,
    mut extracted_writer: MessageWriter<ExtractionCompletedMessage>,
    survivor: Single<(Entity, &Transform), With<Survivor>>,
    mut zone: Single<(&mut ExtractionZone, &Transform)>,
) {
    let dt = time.delta_secs();
    let (ref mut zone_data, zone_tf) = *zone;

    if zone_data.completed {
        return;
    }

    let (survivor_entity, survivor_tf) = *survivor;
    let dist = Vec2::new(
        survivor_tf.translation.x - zone_tf.translation.x,
        survivor_tf.translation.z - zone_tf.translation.z,
    )
    .length();
    let inside_survivor = (dist <= zone_data.radius).then_some(survivor_entity);

    zone_data.active = inside_survivor.is_some();
    if let Some(survivor) = inside_survivor {
        zone_data.progress_secs = (zone_data.progress_secs + dt).min(zone_data.hold_duration_secs);
        if zone_data.progress_secs >= zone_data.hold_duration_secs {
            zone_data.completed = true;
            extracted_writer.write(ExtractionCompletedMessage { survivor });
        }
    }

    stats.in_extraction_zone = zone_data.active;
    stats.extraction_progress_secs = zone_data.progress_secs;
    stats.extraction_hold_secs = zone_data.hold_duration_secs;
}


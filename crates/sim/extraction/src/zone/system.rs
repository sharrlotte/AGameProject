use bevy::prelude::*;
use data::{
    ConfigHotReloadedMessage, ExtractionCompletedMessage, RunEntity, RunStats, GameScene,
};
use world::Survivor;

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
    mut zones: Query<(&mut ExtractionZone, &mut Transform)>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }

    for (mut zone, mut transform) in &mut zones {
        zone.radius = cfg.pad_radius;
        zone.hold_duration_secs = cfg.hold_duration_secs;
        transform.translation = cfg.pad_center;
    }
}

pub fn update_extraction_zone(
    time: Res<Time>,
    mut stats: ResMut<RunStats>,
    mut extracted_writer: MessageWriter<ExtractionCompletedMessage>,
    survivors: Query<(Entity, &Transform), With<Survivor>>,
    mut zones: Query<(&mut ExtractionZone, &Transform)>,
) {
    let dt = time.delta_secs();

    for (mut zone, zone_tf) in &mut zones {
        if zone.completed {
            continue;
        }

        let inside_survivor = survivors.iter().find_map(|(entity, tf)| {
            let dist = Vec2::new(
                tf.translation.x - zone_tf.translation.x,
                tf.translation.z - zone_tf.translation.z,
            )
            .length();
            (dist <= zone.radius).then_some(entity)
        });

        zone.active = inside_survivor.is_some();
        if let Some(survivor) = inside_survivor {
            zone.progress_secs = (zone.progress_secs + dt).min(zone.hold_duration_secs);
            if zone.progress_secs >= zone.hold_duration_secs {
                zone.completed = true;
                extracted_writer.write(ExtractionCompletedMessage { survivor });
            }
        }

        stats.in_extraction_zone = zone.active;
        stats.extraction_progress_secs = zone.progress_secs;
        stats.extraction_hold_secs = zone.hold_duration_secs;
    }
}

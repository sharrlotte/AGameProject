use bevy::prelude::*;
use data::{
    ConfigHotReloadedMessage, DeployRequestedMessage, EnemyKilledMessage, EnemyKind,
    ExtractionCompletedMessage, ExtractionConfig, GameState, LastRunOutcome,
    OpenUpgradesRequestedMessage, PlayerDiedMessage, PurchaseUpgradeMessage, RunEntity, RunStats,
    ScrapBank, ScrapCollectedMessage, SimPostPhysicsSet, SimUpdateSet, UpgradeLevels,
    UpgradeModifiers, UpgradesChangedMessage, UpgradesConfig,
};
use sim_world::Survivor;

#[derive(Component, Debug, Clone, Copy)]
pub struct ScrapPickup {
    pub amount: u32,
    pub radius: f32,
    pub lifetime: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ExtractionZone {
    pub radius: f32,
    pub hold_duration_secs: f32,
    pub progress_secs: f32,
    pub active: bool,
    pub completed: bool,
}

pub struct ExtractionPlugin;

impl Plugin for ExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_extraction_zone)
            .add_systems(
                Update,
                (
                    sync_extraction_hot_reload,
                    collect_scrap_pickups,
                    update_extraction_zone,
                )
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                spawn_scrap_on_enemy_killed
                    .in_set(SimPostPhysicsSet)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub struct RunFlowPlugin;

impl Plugin for RunFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), reset_run_stats)
            .add_systems(OnExit(GameState::Playing), cleanup_run_entities)
            .add_systems(
                Update,
                tick_run_timer
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                (
                    accumulate_kills_on_event,
                    accumulate_scrap_on_event,
                    handle_player_died_event,
                    handle_extraction_completed_event,
                )
                    .after(SimPostPhysicsSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                handle_navigation_requests
                    .run_if(|state: Res<State<GameState>>| {
                        matches!(state.get(), GameState::GameOver | GameState::Upgrades)
                    }),
            );
    }
}

pub struct UpgradesPlugin;

impl Plugin for UpgradesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_upgrade_purchases.run_if(in_state(GameState::Upgrades)),
                recompute_upgrades_on_hot_reload,
            ),
        );
    }
}

fn spawn_extraction_zone(mut commands: Commands, cfg: Res<ExtractionConfig>) {
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
    ));
}

fn sync_extraction_hot_reload(
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

fn spawn_scrap_on_enemy_killed(
    mut commands: Commands,
    cfg: Res<ExtractionConfig>,
    mut killed_reader: MessageReader<EnemyKilledMessage>,
) {
    for msg in killed_reader.read() {
        let amount = match msg.kind {
            EnemyKind::Walker => cfg.walker_scrap,
            EnemyKind::Runner => cfg.runner_scrap,
        };
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
        ));
    }
}

fn collect_scrap_pickups(
    mut commands: Commands,
    time: Res<Time>,
    cfg: Res<ExtractionConfig>,
    mut scrap_writer: MessageWriter<ScrapCollectedMessage>,
    survivors: Query<(Entity, &Transform), With<Survivor>>,
    mut pickups: Query<(Entity, &mut ScrapPickup, &mut Transform), Without<Survivor>>,
) {
    let dt = time.delta_secs();
    let survivor_positions: Vec<(Entity, Vec3)> =
        survivors.iter().map(|(e, t)| (e, t.translation)).collect();

    for (pickup_entity, mut pickup, mut transform) in &mut pickups {
        pickup.lifetime -= dt;
        if pickup.lifetime <= 0.0 {
            commands.entity(pickup_entity).despawn();
            continue;
        }

        let pos = transform.translation;
        let Some(&(collector, target)) = survivor_positions.iter().min_by(|a, b| {
            let da = Vec2::new(a.1.x - pos.x, a.1.z - pos.z).length_squared();
            let db = Vec2::new(b.1.x - pos.x, b.1.z - pos.z).length_squared();
            da.total_cmp(&db)
        }) else {
            continue;
        };

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

fn update_extraction_zone(
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

fn reset_run_stats(mut stats: ResMut<RunStats>, extraction_cfg: Res<ExtractionConfig>) {
    *stats = RunStats {
        extraction_hold_secs: extraction_cfg.hold_duration_secs,
        ..default()
    };
}

fn cleanup_run_entities(mut commands: Commands, run_entities: Query<Entity, With<RunEntity>>) {
    for entity in &run_entities {
        commands.entity(entity).despawn();
    }
}

fn tick_run_timer(time: Res<Time>, mut stats: ResMut<RunStats>) {
    stats.elapsed_secs += time.delta_secs();
}

fn accumulate_kills_on_event(
    mut killed_reader: MessageReader<EnemyKilledMessage>,
    mut stats: ResMut<RunStats>,
) {
    for _ in killed_reader.read() {
        stats.kills += 1;
    }
}

fn accumulate_scrap_on_event(
    mut scrap_reader: MessageReader<ScrapCollectedMessage>,
    mut stats: ResMut<RunStats>,
) {
    for msg in scrap_reader.read() {
        stats.run_scrap += msg.amount;
    }
}

fn handle_player_died_event(
    mut died_reader: MessageReader<PlayerDiedMessage>,
    mut stats: ResMut<RunStats>,
    mut outcome: ResMut<LastRunOutcome>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if died_reader.read().next().is_some() {
        *outcome = LastRunOutcome::Died {
            scrap_lost: stats.run_scrap,
            kills: stats.kills,
            elapsed_secs: stats.elapsed_secs,
        };
        stats.run_scrap = 0;
        next_state.set(GameState::GameOver);
    }
}

fn handle_extraction_completed_event(
    mut extracted_reader: MessageReader<ExtractionCompletedMessage>,
    stats: Res<RunStats>,
    mut bank: ResMut<ScrapBank>,
    mut outcome: ResMut<LastRunOutcome>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if extracted_reader.read().next().is_some() {
        bank.banked += stats.run_scrap;
        *outcome = LastRunOutcome::Extracted {
            scrap_banked: stats.run_scrap,
            kills: stats.kills,
            elapsed_secs: stats.elapsed_secs,
        };
        next_state.set(GameState::Upgrades);
    }
}

fn handle_navigation_requests(
    mut deploy_reader: MessageReader<DeployRequestedMessage>,
    mut upgrades_reader: MessageReader<OpenUpgradesRequestedMessage>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if deploy_reader.read().next().is_some() {
        next_state.set(GameState::Playing);
    } else if upgrades_reader.read().next().is_some() {
        next_state.set(GameState::Upgrades);
    }
}

fn handle_upgrade_purchases(
    mut purchase_reader: MessageReader<PurchaseUpgradeMessage>,
    cfg: Res<UpgradesConfig>,
    mut bank: ResMut<ScrapBank>,
    mut levels: ResMut<UpgradeLevels>,
    mut modifiers: ResMut<UpgradeModifiers>,
    mut changed_writer: MessageWriter<UpgradesChangedMessage>,
) {
    let mut bought = false;
    for msg in purchase_reader.read() {
        let lvl = levels.level(msg.kind);
        let max_lvl = levels.max_level(msg.kind, &cfg);
        let cost = levels.next_cost(msg.kind, &cfg);
        if lvl < max_lvl && bank.banked >= cost {
            bank.banked -= cost;
            levels.increment(msg.kind);
            bought = true;
        }
    }

    if bought {
        modifiers.recompute(&levels, &cfg);
        changed_writer.write(UpgradesChangedMessage);
    }
}

fn recompute_upgrades_on_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<UpgradesConfig>,
    levels: Res<UpgradeLevels>,
    mut modifiers: ResMut<UpgradeModifiers>,
) {
    if reloaded.read().next().is_some() && cfg.is_changed() {
        modifiers.recompute(&levels, &cfg);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use data::UpgradeKind;

    #[test]
    fn extraction_hold_pauses_outside_zone_and_completes_at_duration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ExtractionConfig>()
            .init_resource::<RunStats>()
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(0.25),
            ))
            .add_message::<ExtractionCompletedMessage>()
            .add_systems(Update, update_extraction_zone);

        let survivor = app
            .world_mut()
            .spawn((Survivor, Transform::from_xyz(0.0, 0.9, -19.5)))
            .id();
        app.world_mut().spawn((
            ExtractionZone {
                radius: 3.2,
                hold_duration_secs: 4.0,
                progress_secs: 0.0,
                active: false,
                completed: false,
            },
            Transform::from_xyz(0.0, 0.0, -19.5),
        ));

        app.update(); // initialize TimePlugin baseline (dt = 0)
        for _ in 0..8 {
            app.update(); // 8 * 0.25s = 2.0s
        }
        assert!((app.world().resource::<RunStats>().extraction_progress_secs - 2.0).abs() < 0.01);

        // Move survivor out -> progress pauses instead of resetting
        app.world_mut()
            .get_mut::<Transform>(survivor)
            .unwrap()
            .translation = Vec3::new(0.0, 0.9, 0.0);
        for _ in 0..4 {
            app.update();
        }
        assert!((app.world().resource::<RunStats>().extraction_progress_secs - 2.0).abs() < 0.01);

        // Return inside -> completes at 4.0s
        app.world_mut()
            .get_mut::<Transform>(survivor)
            .unwrap()
            .translation = Vec3::new(0.0, 0.9, -19.5);
        for _ in 0..8 {
            app.update();
        }
        assert!((app.world().resource::<RunStats>().extraction_progress_secs - 4.0).abs() < 0.01);

        let msgs = app.world().resource::<Messages<ExtractionCompletedMessage>>();
        let mut cursor = msgs.get_cursor();
        let completed: Vec<_> = cursor.read(msgs).copied().collect();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].survivor, survivor);
    }

    #[test]
    fn upgrade_purchase_deducts_scrap_and_updates_modifiers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<UpgradesConfig>()
            .insert_resource(ScrapBank { banked: 25 })
            .init_resource::<UpgradeLevels>()
            .init_resource::<UpgradeModifiers>()
            .add_message::<PurchaseUpgradeMessage>()
            .add_message::<UpgradesChangedMessage>()
            .add_message::<ConfigHotReloadedMessage>()
            .add_systems(Update, handle_upgrade_purchases);

        app.world_mut().write_message(PurchaseUpgradeMessage {
            kind: UpgradeKind::WeaponDamage,
        });
        app.update();

        assert_eq!(app.world().resource::<ScrapBank>().banked, 15);
        assert_eq!(app.world().resource::<UpgradeLevels>().weapon_damage, 1);
        assert!((app.world().resource::<UpgradeModifiers>().damage_multiplier - 1.15).abs() < 0.001);
    }
}

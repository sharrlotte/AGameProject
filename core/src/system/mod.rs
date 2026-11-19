use bevy::post_process::bloom::BloomCompositeMode;
use bevy::prelude::*;
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
};

use crate::config::GameConfigValue;
use crate::resource::DeployPayload;
use crate::save::design::ShipCatalog;
use crate::save::fleet::load_or_seed_fleet;


pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.3,
            low_frequency_boost: 0.5,
            low_frequency_boost_curvature: 0.5,
            high_pass_frequency: 1.0,
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        },
        DebandDither::Enabled,
        SpatialListener::new(400.0),
    ));
}

pub fn load_default_fleet(
    mut deploy: ResMut<DeployPayload>,
    catalog: Res<ShipCatalog>,
    config: Res<GameConfigValue>,
) {
    deploy.fleet = load_or_seed_fleet(&config.0, &catalog);
}

#[cfg(debug_assertions)]
pub mod debug_audit {
    use super::*;
    use crate::resource::{PauseState, SceneState, SettingState};
    use crate::simulation::SimulationSet;
    use bevy::time::TimerMode;

    pub struct DebugAuditPlugin;

    impl Plugin for DebugAuditPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<SceneAuditTimer>()
                .add_systems(Update, audit_scene_entities.in_set(SimulationSet::Present));
        }
    }

    #[derive(Resource)]
    struct SceneAuditTimer(Timer);

    impl Default for SceneAuditTimer {
        fn default() -> Self {
            Self(Timer::from_seconds(5.0, TimerMode::Repeating))
        }
    }

    fn audit_scene_entities(
        time: Res<Time>,
        mut timer: ResMut<SceneAuditTimer>,
        roots: Query<(Entity, &GlobalTransform), (Without<ChildOf>, Without<Camera2d>)>,
        scene_tagged: Query<&DespawnOnExit<SceneState>>,
        pause_tagged: Query<&DespawnOnExit<PauseState>>,
        setting_tagged: Query<&DespawnOnExit<SettingState>>,
    ) {
        if !timer.0.tick(time.delta()).just_finished() {
            return;
        }

        let mut tagged = 0usize;
        let mut untagged = Vec::new();

        for (entity, _) in &roots {
            if scene_tagged.contains(entity)
                || pause_tagged.contains(entity)
                || setting_tagged.contains(entity)
            {
                tagged += 1;
            } else {
                untagged.push(entity);
            }
        }

        if !untagged.is_empty() {
            warn!(
                "scene audit: {} state-scoped roots, {} untagged roots: {:?}",
                tagged,
                untagged.len(),
                untagged
            );
        }
    }
}

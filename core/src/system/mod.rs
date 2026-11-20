use bevy::camera::ScalingMode;
use bevy::post_process::bloom::BloomCompositeMode;
use bevy::prelude::*;
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
};

use crate::config::GameConfigValue;
use crate::constant::VIEW_HEIGHT;
use crate::resource::DeployPayload;
use crate::save::design::{ShipCatalog, starter_fleet};


pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
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
    deploy.fleet = starter_fleet(&config.0, catalog.designs());
}

#[cfg(debug_assertions)]
pub mod debug_audit {
    use super::*;
    use crate::resource::SceneState;
    use crate::simulation::SimulationSet;
    use crate::ui::ModalRoot;
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
        modal_tagged: Query<&ModalRoot>,
    ) {
        if !timer.0.tick(time.delta()).just_finished() {
            return;
        }

        let mut tagged = 0usize;
        let mut untagged = Vec::new();

        for (entity, _) in &roots {
            if scene_tagged.contains(entity) || modal_tagged.contains(entity) {
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

#[cfg(test)]
mod tests {
    use bevy::camera::{CameraProjection, ScalingMode};
    use bevy::prelude::*;

    use crate::constant::VIEW_HEIGHT;

    fn projection_area(width: f32, height: f32) -> Vec2 {
        let mut projection = OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        };
        projection.update(width, height);
        projection.area.size()
    }

    #[test]
    fn camera_fixed_vertical_preserves_design_frame() {
        assert_eq!(projection_area(1280.0, 720.0), Vec2::new(1920.0, 1080.0));
        assert_eq!(projection_area(2560.0, 1080.0), Vec2::new(2560.0, 1080.0));
        assert_eq!(projection_area(1440.0, 1080.0), Vec2::new(1440.0, 1080.0));
    }
}

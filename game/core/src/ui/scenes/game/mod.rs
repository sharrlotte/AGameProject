mod background;

use std::time::Duration;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::content::bullet::base::Bullet;
use crate::content::ship::base::{PlayerShipController, Ship};
use crate::ui::scenes::game::background::{BackgroundConfig, BackgroundImage, BackgroundPlugin};
use crate::{GameState, SceneState, TurretControl};

#[derive(Resource, Default)]
struct GameInitialized(bool);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameInitialized>()
            .add_systems(OnEnter(SceneState::Game), setup_game)
            .add_systems(
                OnExit(SceneState::Game),
                (cleanup_game, cleanup_game_entities),
            )
            .add_systems(
                Update,
                (
                    camera_follow,
                    camera_zoom,
                    update_turret_control_mode_button,
                    update_fps.run_if(on_timer(Duration::from_millis(100))),
                )
                    .run_if(in_state(SceneState::Game)),
            )
            .add_plugins(BackgroundPlugin);
    }
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct TurretControlModeButton;

#[derive(Component)]
struct TurretControlModeText;

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut text: Single<&mut Text, With<FpsText>>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            text.0 = format!("FPS: {value:.2}",);
        }
    }
}

fn setup_game(
    mut commands: Commands,
    mut initialized: ResMut<GameInitialized>,
    game_state: Res<GameState>,
    fps_query: Query<(), With<FpsText>>,
) {
    if initialized.0 {
        // Only spawn FpsText if not exists
        if fps_query.is_empty() {
            commands.spawn((
                Text::new("FPS: N/A"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                FpsText,
            ));
        }
        return;
    }
    initialized.0 = true;

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        })
        .with_children(|container| {
            let text = match game_state.turret_control {
                TurretControl::Manual => "Turret Control: Manual",
                TurretControl::Auto => "Turret Control: Automatic",
                TurretControl::Hold => "Turret Control: Hold",
            };

            container
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    TurretControlModeButton,
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(text),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TurretControlModeText,
                    ));
                });

            container.spawn((
                Text::new("FPS: N/A"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node { ..default() },
                FpsText,
            ));
        });
}

fn update_turret_control_mode_button(
    mut game_state: ResMut<GameState>,
    mut button_query: Query<&Interaction, (Changed<Interaction>, With<TurretControlModeButton>)>,
    mut text_query: Query<&mut Text, With<TurretControlModeText>>,
) {
    let Ok(interaction) = button_query.single_mut() else {
        return;
    };

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    if *interaction == Interaction::Pressed {
        game_state.turret_control = game_state.turret_control.next();

        let mod_string = match game_state.turret_control {
            TurretControl::Manual => "Turret Control: Manual",
            TurretControl::Auto => "Turret Control: Automatic",
            TurretControl::Hold => "Turret Control: Hold",
        };
        text.0 = mod_string.to_string();
    }
}

fn cleanup_game(mut commands: Commands, query: Query<Entity, With<FpsText>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_game_entities(
    mut commands: Commands,
    ships: Query<Entity, With<Ship>>,
    bullets: Query<Entity, With<Bullet>>,
    backgrounds: Query<Entity, With<BackgroundImage>>,
    mut initialized: ResMut<GameInitialized>,
) {
    initialized.0 = false;

    for entity in &ships {
        commands.entity(entity).despawn();
    }
    for entity in &bullets {
        commands.entity(entity).despawn();
    }
    for entity in &backgrounds {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<BackgroundConfig>();
}

fn camera_follow(
    player_query: Query<(&Transform, &Ship), (With<PlayerShipController>, Without<Camera2d>)>,
    mut camera_transform: Single<&mut Transform, (With<Camera2d>, Without<PlayerShipController>)>,
    time: Res<Time>,
) {
    if player_query.is_empty() {
        return;
    }

    let (ship_transform, ship) = player_query.iter().next().unwrap();

    let target = ship_transform
        .transform_point(ship.center.extend(0.0))
        .truncate();

    let current = camera_transform.translation.truncate();

    let smooth_speed = 10.0;
    let t = 1.0 - (-smooth_speed * time.delta_secs());
    let new_position = current.lerp(target, t);

    camera_transform.translation.x = new_position.x;
    camera_transform.translation.y = new_position.y;
}

fn camera_zoom(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
) {
    for event in mouse_wheel_events.read() {
        for mut projection in camera_query.iter_mut() {
            if let Projection::Orthographic(ortho) = &mut *projection {
                let zoom_speed = 0.1;
                ortho.scale -= event.y * zoom_speed * ortho.scale;
                ortho.scale = ortho.scale.clamp(0.1, 5.0);
            }
        }
    }
}

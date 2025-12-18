mod background;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::AppState;
use crate::content::bullet::types::Bullet;
use crate::content::ships::{PlayerShipController, ShipPlugin};
use crate::content::structures::ship::Ship;
use crate::ui::scenes::game::background::{BackgroundConfig, BackgroundImage, BackgroundPlugin};

#[derive(Resource, Default)]
struct GameInitialized(bool);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameInitialized>()
            .add_systems(OnEnter(AppState::Game), setup_game)
            .add_systems(
                OnExit(AppState::Game),
                (cleanup_game, cleanup_game_entities),
            )
            .add_systems(
                Update,
                (camera_follow, camera_zoom, update_fps, handle_pause_input)
                    .run_if(in_state(AppState::Game)),
            )
            .add_plugins(ShipPlugin)
            .add_plugins(BackgroundPlugin);
    }
}

#[derive(Component)]
struct FpsText;

fn update_fps(time: Res<Time>, mut text: Single<&mut Text, With<FpsText>>) {
    let fps = 1.0 / time.delta_secs();
    if fps.is_finite() {
        text.0 = format!("FPS: {:.0}", fps);
    }
}

fn setup_game(
    mut commands: Commands,
    mut initialized: ResMut<GameInitialized>,
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

    info!("Setting up Game Scene");

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

fn cleanup_game(mut commands: Commands, query: Query<Entity, With<FpsText>>) {
    info!("Cleaning up Game Scene UI");
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
    info!("Cleaning up Game Entities");
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
    player: Single<(&Transform, &Ship), (With<PlayerShipController>, Without<Camera2d>)>,
    mut camera_transform: Single<&mut Transform, (With<Camera2d>, Without<PlayerShipController>)>,
    time: Res<Time>,
) {
    let (ship_transform, ship) = *player;

    let target = ship_transform
        .transform_point(ship.center.extend(0.0))
        .truncate();

    let current = camera_transform.translation.truncate();

    // Smooth follow
    let smooth_speed = 5.0;
    let new_pos = current.lerp(target, time.delta_secs() * smooth_speed);

    camera_transform.translation.x = new_pos.x;
    camera_transform.translation.y = new_pos.y;
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

fn handle_pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        info!("Pausing game");
        next_state.set(AppState::Pause);
    }
}

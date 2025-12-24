use bevy::prelude::*;

use crate::SceneState;
use crate::scene::system::clean_scene_items;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Game), setup)
            .add_systems(Update, parallax_follow.run_if(in_state(SceneState::Game)))
            .add_systems(OnExit(SceneState::Home), clean_scene_items);
    }
}

#[derive(Component)]
struct ParallaxLayer {
    factor: f32,  // movement multiplier (0.05..1.0)
    radius: f32,  // wrap distance
    offset: Vec2, // fixed random offset
}

fn spawn_parallax_layer(
    commands: &mut Commands,
    count: usize,
    factor: f32,
    radius: f32,
    size: f32,
    color: Color,
    seed: u64,
) {
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    let mut rng = SmallRng::seed_from_u64(seed);

    for _ in 0..count {
        let offset = Vec2::new(
            rng.random_range(-radius..radius),
            rng.random_range(-radius..radius),
        );

        commands.spawn((
            ParallaxLayer {
                factor,
                radius,
                offset,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::default(),
        ));
    }
}

fn setup(mut commands: Commands) {
    // FAR stars
    spawn_parallax_layer(
        &mut commands,
        600,
        0.01,
        8000.0,
        2.0,
        Color::srgb(0.7, 0.7, 0.8),
        1,
    );

    // MID dust
    spawn_parallax_layer(
        &mut commands,
        300,
        0.03,
        6000.0,
        3.2,
        Color::srgb(0.8, 0.8, 0.8),
        2,
    );

    // NEAR debris
    spawn_parallax_layer(
        &mut commands,
        120,
        0.06,
        5000.0,
        4.5,
        Color::srgb(1.0, 1.0, 1.0),
        3,
    );
}

fn parallax_follow(
    camera: Query<&Transform, With<Camera2d>>,
    mut layers: Query<(&ParallaxLayer, &mut Transform), Without<Camera2d>>,
) {
    let Ok(transform) = camera.single() else {
        return;
    };

    let position = transform.translation.truncate();

    for (layer, mut transform) in &mut layers {
        let mut pos = position * layer.factor + layer.offset;

        let r = layer.radius;
        pos.x = wrap(pos.x, position.x - r, position.x + r);
        pos.y = wrap(pos.y, position.y - r, position.y + r);

        transform.translation = pos.extend(-layer.factor * 10.0);
    }
}

fn wrap(value: f32, min: f32, max: f32) -> f32 {
    let range = max - min;
    value - ((value - min) / range).floor() * range
}

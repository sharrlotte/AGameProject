use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::AppState;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), setup_background)
            .add_systems(Update, update_background.run_if(in_state(AppState::Game)))
            .add_systems(OnExit(AppState::Game), cleanup_background);
    }
}

#[derive(Component)]
pub struct BackgroundImage;

#[derive(Resource)]
pub struct BackgroundConfig;

fn setup_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the single background image
    let texture_handle = asset_server.load("textures/background/background1.png");

    commands.insert_resource(BackgroundConfig);

    // Spawn a single sprite for the background
    commands.spawn((
        Sprite {
            image: texture_handle,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -100.0), // Behind everything
        BackgroundImage,
    ));
}

fn cleanup_background(mut commands: Commands, query: Query<Entity, With<BackgroundImage>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<BackgroundConfig>();
}

fn update_background(
    bg_transform: Single<(&mut Transform, &Sprite), (With<BackgroundImage>, Without<Camera2d>)>,
    camera_query: Single<(&Transform, &Projection), With<Camera2d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
) {
    let (mut transform, sprite) = bg_transform.into_inner();
    let (camera_transform, projection) = *camera_query;
    let camera_pos = camera_transform.translation.truncate();

    // Get zoom level from projection
    let zoom = if let Projection::Orthographic(ortho) = projection {
        ortho.scale
    } else {
        1.0
    };

    // --- Parallax Movement ---
    // User Requirement: "move it slowly base on user positon, user max position is x=50k and y=50k, lerp it base on that"
    // We map the player's position to a background position.
    // If the background moves at 95% of the camera speed, it appears very far away.
    // Relative displacement = CameraPos * (1 - Factor).
    // At 50,000 units, with 0.95 factor, relative displacement is 2,500 units.
    let parallax_factor = 0.95;
    let target_pos = camera_pos * parallax_factor;

    // --- Zoom Scaling ---
    // User Requirement: "scale it when use zoom too so when user zoom, they only see it bigger a bit"
    // Normal behavior: VisualSize ~ 1/Zoom.
    // We want it to be less sensitive.
    // If we scale the object by sqrt(Zoom), then VisualSize ~ sqrt(Zoom)/Zoom = 1/sqrt(Zoom).
    // This dampens the zoom effect.
    let zoom_damping = zoom.sqrt();

    // User Requirement: "make it 2x bigger than user viewport is enough"
    // Calculate required base scale dynamically
    let mut base_scale = 10.0; // Default fallback

    if let Some(image) = images.get(&sprite.image) {
        let image_size = Vec2::new(image.width() as f32, image.height() as f32);
        let window_size = Vec2::new(window.width(), window.height());

        if image_size.x > 0.0 && image_size.y > 0.0 {
            // Determine scale needed to cover the window exactly at 1:1 zoom
            let width_ratio = window_size.x / image_size.x;
            let height_ratio = window_size.y / image_size.y;
            let cover_scale = width_ratio.max(height_ratio);

            // We want 2x viewport coverage at Max Zoom (5.0)
            // Logic:
            // FinalScale = BaseScale * ZoomDamping
            // FinalScale(at MaxZoom) >= CoverScale * MaxZoom * 2.0
            // BaseScale * sqrt(MaxZoom) >= CoverScale * MaxZoom * 2.0
            // BaseScale >= CoverScale * MaxZoom * 2.0 / sqrt(MaxZoom)
            // BaseScale >= CoverScale * sqrt(MaxZoom) * 2.0

            // Max Zoom is 5.0. sqrt(5) approx 2.236
            // BaseScale >= CoverScale * 2.236 * 2.0 = CoverScale * 4.47

            // We use 4.5 as a safe multiplier.
            base_scale = cover_scale * 4.5;
        }
    }

    transform.translation.x = target_pos.x;
    transform.translation.y = target_pos.y;
    transform.translation.z = -100.0;

    transform.scale = Vec3::new(base_scale * zoom_damping, base_scale * zoom_damping, 1.0);
}

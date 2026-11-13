use bevy::prelude::*;

#[derive(Resource)]
pub struct SharedVisualMaterials {
    pub ground: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub cover: Handle<StandardMaterial>,
    pub walker: Handle<StandardMaterial>,
    pub runner: Handle<StandardMaterial>,
    pub projectile: Handle<StandardMaterial>,
    pub hit_flash: Handle<StandardMaterial>,
    pub scrap: Handle<StandardMaterial>,
    pub extraction_idle: Handle<StandardMaterial>,
    pub extraction_active: Handle<StandardMaterial>,
}

impl FromWorld for SharedVisualMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self {
            ground: materials.add(StandardMaterial {
                base_color: Color::srgb(0.14, 0.16, 0.19),
                perceptual_roughness: 0.9,
                ..default()
            }),
            wall: materials.add(StandardMaterial {
                base_color: Color::srgb(0.26, 0.29, 0.34),
                perceptual_roughness: 0.85,
                ..default()
            }),
            cover: materials.add(StandardMaterial {
                base_color: Color::srgb(0.55, 0.45, 0.30),
                perceptual_roughness: 0.75,
                ..default()
            }),
            walker: materials.add(StandardMaterial {
                base_color: Color::srgb(0.36, 0.62, 0.30),
                perceptual_roughness: 0.7,
                ..default()
            }),
            runner: materials.add(StandardMaterial {
                base_color: Color::srgb(0.90, 0.30, 0.22),
                perceptual_roughness: 0.6,
                ..default()
            }),
            projectile: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.88, 0.35),
                emissive: LinearRgba::rgb(4.0, 3.0, 0.6),
                ..default()
            }),
            hit_flash: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::rgb(3.5, 3.5, 3.5),
                ..default()
            }),
            scrap: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.74, 0.18),
                emissive: LinearRgba::rgb(1.8, 1.1, 0.15),
                metallic: 0.5,
                perceptual_roughness: 0.3,
                ..default()
            }),
            extraction_idle: materials.add(StandardMaterial {
                base_color: Color::srgba(0.15, 0.82, 0.75, 0.65),
                emissive: LinearRgba::rgb(0.2, 1.2, 1.0),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            extraction_active: materials.add(StandardMaterial {
                base_color: Color::srgba(0.25, 1.0, 0.55, 0.85),
                emissive: LinearRgba::rgb(0.5, 2.8, 1.2),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        }
    }
}








use bevy::prelude::*;

/// Armored Tank materials: steel plate, rusted trim, dark under-suit, and the exposed back.
#[derive(Debug, Clone)]
pub struct TankMaterials {
    pub steel: Handle<StandardMaterial>,
    pub rust: Handle<StandardMaterial>,
    pub suit: Handle<StandardMaterial>,
    pub weak: Handle<StandardMaterial>,
}

#[derive(Debug, Clone)]
pub struct SpitterMaterials {
    pub skin: Handle<StandardMaterial>,
    pub sac: Handle<StandardMaterial>,
    pub rag: Handle<StandardMaterial>,
}

#[derive(Debug, Clone)]
pub struct ExploderMaterials {
    pub skin: Handle<StandardMaterial>,
    pub pustule: Handle<StandardMaterial>,
    pub crack: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct SharedVisualMaterials {
    pub ground: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub cover: Handle<StandardMaterial>,
    pub walker: Handle<StandardMaterial>,
    pub runner: Handle<StandardMaterial>,
    pub tank: TankMaterials,
    pub spitter: SpitterMaterials,
    pub exploder: ExploderMaterials,
    pub projectile: Handle<StandardMaterial>,
    pub hit_flash: Handle<StandardMaterial>,
    pub acid: Handle<StandardMaterial>,
    pub blast: Handle<StandardMaterial>,
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
            tank: TankMaterials {
                steel: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0x3E, 0x46, 0x50),
                    metallic: 0.65,
                    perceptual_roughness: 0.45,
                    ..default()
                }),
                rust: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0x8A, 0x5A, 0x32),
                    metallic: 0.10,
                    perceptual_roughness: 0.80,
                    ..default()
                }),
                suit: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0x24, 0x27, 0x2C),
                    perceptual_roughness: 0.90,
                    ..default()
                }),
                weak: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0x7A, 0x3B, 0x32),
                    perceptual_roughness: 0.70,
                    ..default()
                }),
            },
            spitter: SpitterMaterials {
                skin: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0x8A, 0x6B, 0x4A),
                    perceptual_roughness: 0.80,
                    ..default()
                }),
                sac: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0xB6, 0xFF, 0x4A),
                    emissive: LinearRgba::rgb(0.45, 1.8, 0.12),
                    perceptual_roughness: 0.25,
                    ..default()
                }),
                rag: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0x4A, 0x44, 0x38),
                    perceptual_roughness: 0.90,
                    ..default()
                }),
            },
            exploder: ExploderMaterials {
                skin: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0xA8, 0x5A, 0x3C),
                    perceptual_roughness: 0.80,
                    ..default()
                }),
                pustule: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0xFF, 0x7A, 0x2E),
                    emissive: LinearRgba::rgb(2.0, 0.55, 0.10),
                    perceptual_roughness: 0.20,
                    ..default()
                }),
                crack: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(0x3A, 0x1E, 0x16),
                    perceptual_roughness: 0.90,
                    ..default()
                }),
            },
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
            acid: materials.add(StandardMaterial {
                base_color: Color::srgba(0.62, 0.95, 0.18, 0.55),
                emissive: LinearRgba::rgb(0.45, 1.6, 0.20),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.35,
                ..default()
            }),
            blast: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.58, 0.16, 0.65),
                emissive: LinearRgba::rgb(4.0, 1.6, 0.32),
                alpha_mode: AlphaMode::Blend,
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

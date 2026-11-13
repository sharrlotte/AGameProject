use bevy::prelude::*;
use crate::message::messages::WeaponFiredMessage;

/// Short-lived muzzle flash spawned at the projectile origin.
#[derive(Component, Debug)]
pub struct MuzzleFlash {
    pub timer: f32,
}

const FLASH_LIFETIME: f32 = 0.08;

/// Spawns a brief flash entity at each fired weapon's muzzle origin.
pub fn spawn_muzzle_flash(
    mut commands: Commands,
    mut fired: MessageReader<WeaponFiredMessage>,
) {
    for message in fired.read() {
        commands.spawn((
            MuzzleFlash {
                timer: FLASH_LIFETIME,
            },
            PointLight {
                intensity: 8000.0,
                range: 4.0,
                color: Color::srgb(1.0, 0.85, 0.5),
                ..default()
            },
            Transform::from_translation(message.origin),
            Visibility::default(),
        ));
    }
}

/// Ticks flash lifetimes and despawns expired flashes.
pub fn update_muzzle_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut MuzzleFlash)>,
) {
    let dt = time.delta_secs();
    for (entity, mut flash) in &mut flashes {
        flash.timer -= dt;
        if flash.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}







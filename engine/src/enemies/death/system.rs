use bevy::prelude::*;
use crate::common::Dying;

/// Despawns enemies whose death effects have finished resolving.
pub fn cleanup_dead(mut commands: Commands, dead: Query<Entity, With<Dying>>) {
    for entity in &dead {
        commands.entity(entity).despawn();
    }
}

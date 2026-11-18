use super::Lifetime;
use bevy::prelude::*;


use crate::content::body::debris::Debris;

pub fn update_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime), Without<Debris>>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.tick(&time);

        if lifetime.timer().is_finished() {
            let Ok(mut commands) = commands.get_entity(entity) else {
                continue;
            };

            commands.despawn();
        }
    }
}

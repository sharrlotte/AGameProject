use bevy::prelude::*;

use crate::common::{death::component::Death, lifetime::component::Lifetime};

pub fn update_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.tick(&time);

        if lifetime.timer().is_finished() {
            let Ok(mut commands) = commands.get_entity(entity) else {
                continue;
            };

            commands.insert(Death);
        }
    }
}

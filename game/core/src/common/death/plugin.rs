use bevy::
    app::{App, FixedLast, Plugin}
;

use crate::common::death::system::despawn_dead;

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedLast, despawn_dead);
    }
}

use bevy::prelude::*;

use crate::content::team::Team;

// TODO: Do we actually need a death component? It seems like we could just despawn the entity when it is destroyed. The only reason to keep it around would be if we wanted to do some kind of death animation or effect, but that should probably be handled by a separate system that listens for the ShipDestroyed message.
#[derive(Component)]
pub struct Death;

#[derive(Message, Debug, Clone, Copy)]
pub struct ShipDestroyed {
    pub ship: Entity,
    pub team: Team,
}

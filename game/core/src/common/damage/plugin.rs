use bevy::prelude::*;

use crate::common::damage::{
    message::DamageEvent,
    system::{clear_damage_text, handle_damage_collisions, handle_damage_events},
};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_systems(FixedUpdate, handle_damage_collisions)
            .add_systems(FixedPostUpdate, (handle_damage_events, clear_damage_text));
    }
}

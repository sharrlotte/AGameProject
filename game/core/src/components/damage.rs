use bevy::{
    app::{App, Plugin, PostUpdate},
    ecs::{
        component::Component,
        entity::Entity,
        message::{Message, MessageReader},
        system::Query,
    },
    reflect::Reflect,
};

use crate::{content::block::base::Health, save::base::Save};

#[derive(Component, Debug)]
#[require(Save)]
pub struct Damage {
    pub damage: f32,
}

#[derive(Message, Debug, Clone, Reflect)]
pub struct DamageEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
}

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_systems(PostUpdate, handle_damage_events);
    }
}

fn handle_damage_events(mut events: MessageReader<DamageEvent>, mut healths: Query<&mut Health>) {
    for event in events.read() {
        if let Ok(mut health) = healths.get_mut(event.target) {
            health.health -= event.damage;
        }
    }
}

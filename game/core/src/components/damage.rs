use bevy::{
    app::{App, Plugin, PostUpdate, Update},
    ecs::{
        component::Component,
        entity::Entity,
        message::{Message, MessageReader, MessageWriter},
        system::Query,
    },
    reflect::Reflect,
};
use bevy_rapier2d::prelude::CollisionEvent;

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
            .add_systems(Update, handle_damage_collisions)
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

pub fn handle_damage_collisions(
    mut collision_events: MessageReader<CollisionEvent>,
    mut writer: MessageWriter<DamageEvent>,
    damages: Query<(Entity, &Damage)>,
    healths: Query<(Entity, &Health)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            if let Ok((damage, damagec)) = damages.get(*e1)
                && let Ok((health, _)) = healths.get(*e2)
            {
                writer.write(DamageEvent {
                    attacker: damage,
                    target: health,
                    damage: damagec.damage,
                });
            } else if let Ok((damage, damagec)) = damages.get(*e2)
                && let Ok((health, _)) = healths.get(*e1)
            {
                writer.write(DamageEvent {
                    attacker: damage,
                    target: health,
                    damage: damagec.damage,
                });
            }
        }
    }
}

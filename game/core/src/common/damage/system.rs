use bevy::ecs::{
    entity::Entity,
    message::{MessageReader, MessageWriter},
    system::Query,
};
use bevy_rapier2d::prelude::CollisionEvent;

use crate::{
    common::damage::{component::Damage, message::DamageEvent},
    content::block::component::Health,
};

pub fn handle_damage_events(
    mut events: MessageReader<DamageEvent>,
    mut healths: Query<&mut Health>,
) {
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

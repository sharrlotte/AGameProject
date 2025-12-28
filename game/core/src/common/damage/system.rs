use bevy::{
    color::Color,
    ecs::{
        entity::Entity,
        message::{MessageReader, MessageWriter},
        system::{Commands, Query},
    },
    sprite::Text2d,
    text::{TextColor, TextFont},
    transform::components::{GlobalTransform, Transform},
    utils::default,
};
use bevy_rapier2d::prelude::CollisionEvent;

use crate::{
    common::{
        damage::{component::Damage, message::DamageEvent},
        lifetime::component::Lifetime,
    },
    content::block::component::Health,
};

pub fn handle_damage_events(
    mut commands: Commands,
    mut events: MessageReader<DamageEvent>,
    mut healths: Query<(&mut Health, &GlobalTransform)>,
) {
    for event in events.read() {
        if let Ok((mut health, transform)) = healths.get_mut(event.target) {
            health.health -= event.damage;

            commands.spawn((
                Transform::from_translation(transform.translation().truncate().extend(1000000.0)),
                Lifetime::seconds(1.5),
                Text2d::new(event.damage.to_string()),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
            ));
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

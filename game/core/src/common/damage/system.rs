use bevy::{
    color::Color,
    ecs::{
        entity::Entity,
        message::{MessageReader, MessageWriter},
        system::{Commands, Query, Res},
    },
    text::{TextColor, TextFont},
    time::{Time, Timer, TimerMode},
    transform::components::GlobalTransform,
    ui::widget::Text,
    utils::default,
};
use bevy_rapier2d::prelude::CollisionEvent;

use crate::{
    common::damage::{
        component::{Damage, DamageText},
        message::DamageEvent,
    },
    content::block::component::Health,
};

pub fn clear_damage_text(
    mut commands: Commands,
    mut damage_texts: Query<(Entity, &mut DamageText)>,
    time: Res<Time>,
) {
    for (entity, mut damage_text) in damage_texts.iter_mut() {
        damage_text.timer.tick(time.delta());
        if damage_text.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_damage_events(
    mut commands: Commands,
    mut events: MessageReader<DamageEvent>,
    mut healths: Query<(&mut Health, &GlobalTransform)>,
) {
    for event in events.read() {
        if let Ok((mut health, transform)) = healths.get_mut(event.target) {
            health.health -= event.damage;

            commands.spawn((
                transform.clone(),
                DamageText {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                },
                Text::new(event.damage.to_string()),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::WHITE),
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

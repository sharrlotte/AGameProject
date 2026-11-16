use bevy::prelude::*;
use bevy_rapier2d::prelude::CollisionEvent;

use crate::config::GameConfigValue;
use crate::{content::block::Health, content::lifetime::Lifetime};
use crate::resource::{SceneEntity, SceneState};
use super::*;

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_systems(FixedUpdate, handle_damage_collisions)
            .add_systems(FixedPostUpdate, handle_damage_events);
    }
}

pub fn handle_damage_events(
    mut commands: Commands,
    mut events: MessageReader<DamageEvent>,
    mut healths: Query<(&mut Health, &GlobalTransform)>,
    config: Res<GameConfigValue>,
) {
    for event in events.read() {
        if let Ok((mut health, transform)) = healths.get_mut(event.target) {
            health.health -= event.damage;

            commands.spawn((
                Transform::from_translation(transform.translation().truncate().extend(1000000.0)),
                Lifetime::seconds(config.0.damage_number_lifetime),
                SceneEntity(SceneState::Game),
                Text2d::new(event.damage.to_string()),
                TextFont {
                    font_size: FontSize::Px(20.0),
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

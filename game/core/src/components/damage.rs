use bevy::{
    app::{App, Plugin, PostUpdate},
    color::Color,
    ecs::{
        component::Component,
        entity::Entity,
        message::{Message, MessageReader, MessageWriter},
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res},
    },
    math::Vec2,
    reflect::Reflect,
    sprite::Sprite,
    time::{Time, Timer, TimerMode},
    utils::default,
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

#[derive(Component, Debug)]
pub struct FlashOverlay {
    timer: Timer,
}

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>().add_systems(
            PostUpdate,
            (
                handle_damage_collisions,
                handle_damage_events,
                handle_flash_overlay,
            )
                .chain(),
        );
    }
}

fn handle_damage_events(
    mut commands: Commands,
    mut events: MessageReader<DamageEvent>,
    mut healths: Query<&mut Health>,
) {
    for event in events.read() {
        if let Ok(mut health) = healths.get_mut(event.target) {
            health.health -= event.damage;

            let overlay = commands
                .spawn((
                    Sprite {
                        color: Color::srgba(10.0, 10.0, 10.0, 0.8),
                        custom_size: Some(Vec2::new(64.0, 64.0)),
                        ..default()
                    },
                    FlashOverlay {
                        timer: Timer::from_seconds(0.1, TimerMode::Once),
                    },
                ))
                .id();

            commands.entity(event.target).add_child(overlay);
        }
    }
}

pub fn handle_flash_overlay(
    mut commands: Commands,
    mut flash_overlays: Query<(&mut FlashOverlay, Entity)>,
    time: Res<Time>,
) {
    for (mut flash_overlay, entity) in flash_overlays.iter_mut() {
        flash_overlay.timer.tick(time.delta());
        if flash_overlay.timer.is_finished() {
            commands.entity(entity).despawn();
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
